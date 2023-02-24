use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::path::Path;
use std::str::from_utf8;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use fasteval::*;
use serde_json::Value;

use super::BulletPool;

#[derive(Default)]
pub struct PatternLoader;

// Load assets (I guess?)
impl AssetLoader for PatternLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let pattern = parse(from_utf8(bytes)?);
            load_context.set_default_asset(LoadedAsset::new(pattern));
            Ok(())
        })
    }

    // I don't like this extension, but I don't know how to get rid of it either...
    fn extensions(&self) -> &[&str] {
        &["pattern.json"]
    }
}

impl PatternLoader {
    // Force-load every pattern script
    // I'm not sure if this is the best way to do it,
    // nor if I should do it to begin with, but...
    pub(crate) fn init_database(
        asset_server: Res<AssetServer>,
        mut patterns: ResMut<PatternDatabase>,
    ) {
        if let Ok(scripts_iter) = asset_server
            .asset_io()
            .read_directory(Path::new("./patterns"))
        {
            for path in scripts_iter {
                if !path.to_str().unwrap().ends_with("pattern.json") {
                    continue;
                }

                let handle = asset_server.load(path.clone());
                patterns.0.insert(
                    path.file_stem()
                        .map(|u| u.to_str().unwrap().split('.').next().unwrap())
                        .and_then(|str| Some(String::from(str)))
                        .unwrap(),
                    handle,
                );
            }
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct BulletContext {
    lifetime: f32,
    position: Vec2,
    rotation: f32,
    angular_velocity: Arc<ExpressionSlab>,
    speed: Arc<ExpressionSlab>,
}

impl Default for BulletContext {
    fn default() -> Self {
        Self {
            lifetime: 10.,
            speed: Arc::new(ExpressionSlab::new(
                fasteval::Instruction::IConst(0.),
                Slab::default(),
            )),
            angular_velocity: Arc::new(ExpressionSlab::new(
                fasteval::Instruction::IConst(0.),
                Slab::default(),
            )),
            position: Vec2::default(),
            rotation: f32::default(),
        }
    }
}

impl BulletContext {
    pub fn new(speed: f32) -> Self {
        Self {
            speed: Arc::new(ExpressionSlab::from(speed.to_string().as_str())),
            ..Default::default()
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct PatternDatabase(pub BTreeMap<String, Handle<Pattern>>);

impl PatternDatabase {
    pub fn get(&self, key: &str) -> Option<Handle<Pattern>> {
        self.0.get(key).and_then(|p| Some(p.clone())).clone()
    }
}

// ParsedPattern assets get generated from .json.pattern files in /assets/scripts/
#[derive(Default, Debug, TypeUuid)]
#[uuid = "1ff044c3-1d98-4b22-a7e2-73a41298ff98"]
pub struct Pattern {
    pub operations: Vec<PatternOp>,
}

impl Pattern {
    fn ring(bullets: Vec<BulletContext>, count: u32, radius: f32) -> Vec<BulletContext> {
        bullets
            .iter()
            .flat_map(|b| {
                (0..count).map(|i| {
                    let rotation = b.rotation + i as f32 / count as f32 * 2. * PI;
                    BulletContext {
                        position: b.position + Vec2::from_angle(rotation) * radius,
                        rotation,
                        ..b.clone()
                    }
                })
            })
            .collect()
    }

    fn arc(bullets: Vec<BulletContext>, count: u32, angle: f32) -> Vec<BulletContext> {
        if count == 1 {
            return bullets;
        }

        let step = angle / (count as f32 - 1.0);
        bullets
            .iter()
            .flat_map(|b| {
                (0..count).map(|i| BulletContext {
                    rotation: b.rotation - angle / 2.0 + step * i as f32,
                    ..b.clone()
                })
            })
            .collect()
    }

    pub fn fire(&self, mut bullet_pools: Query<&mut BulletPool>) {
        let mut bullets = vec![BulletContext::new(60.)];

        for op in self.operations.iter() {
            bullets = match op {
                PatternOp::Ring(count, radius) => Pattern::ring(
                    bullets,
                    count.eval(&mut StrToF64Namespace::from([("t", 0.0)])) as u32,
                    *radius,
                ),
                PatternOp::Arc(count, angle) => Pattern::arc(bullets, *count, *angle),
                PatternOp::Bullet(bullet) => {
                    bullets.iter().for_each(|iter_bullet| {
                        bullet_pools.single_mut().add(
                            bullet.lifetime,
                            iter_bullet.position,
                            iter_bullet.rotation,
                            bullet
                                .speed
                                .clone()
                                .eval(&mut StrToF64Namespace::from([("t", 0.0)])),
                            bullet
                                .angular_velocity
                                .clone()
                                .eval(&mut StrToF64Namespace::from([("t", 0.0)])),
                        );
                    });
                    bullets
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PatternOp {
    Ring(Box<ExpressionSlab>, f32),
    Arc(u32, f32 /* fasteval::Expression */),
    Bullet(BulletContext),
}

pub fn parse(source: &str) -> Pattern {
    let json: serde_json::error::Result<Value> = serde_json::from_str(source);

    let mut value = &match json {
        Err(error) => panic!("Error while parsing pattern: {error}"),
        Ok(t) => t,
    };

    let mut pattern = Pattern::default();

    while !value.is_null() {
        if let Value::String(element_type) = &value["type"] {
            pattern.operations.push(match element_type.as_str() {
                "ring" => PatternOp::Ring(
                    Box::new(parse_expression(&value, "count")),
                    value["radius"].as_f64().unwrap_or(0.) as f32,
                ),
                "arc" => PatternOp::Arc(
                    value["count"].as_u64().expect("No count provided.") as u32,
                    (value["angle"].as_f64().expect("No angle provided.") as f32).to_radians(),
                ),
                "bullet" => PatternOp::Bullet(BulletContext {
                    lifetime: value["lifetime"].as_f64().unwrap_or(10.) as f32,
                    speed: Arc::new(parse_expression(&value, "speed")),
                    angular_velocity: Arc::new(parse_expression(&value, "angular_velocity")),
                    ..Default::default()
                }),
                _ => {
                    panic!("Invalid type in pattern.");
                }
            });
        }

        value = &value["child"];
    }

    pattern
}

fn parse_expression(value: &Value, key: &str) -> ExpressionSlab {
    let binding = value[key].to_string();
    let expression = binding.trim_matches('\"');

    let string = expression;
    let mut slab = Slab::new();
    let expression = fasteval::Parser::new()
        .parse(string, &mut slab.ps)
        .unwrap()
        .from(&slab.ps);

    let expression = expression.compile(&slab.ps, &mut slab.cs);

    ExpressionSlab::new(expression, slab)
}

#[derive(Debug)]
pub struct ExpressionSlab {
    expression: Instruction,
    slab: Slab,
}

impl From<&str> for ExpressionSlab {
    fn from(value: &str) -> Self {
        let mut slab = Slab::new();
        let expression = fasteval::Parser::new()
            .parse(value, &mut slab.ps)
            .unwrap()
            .from(&slab.ps);

        let expression = expression.compile(&slab.ps, &mut slab.cs);

        ExpressionSlab::new(expression, slab)
    }
}

impl ExpressionSlab {
    pub fn new(expression: Instruction, slab: Slab) -> Self {
        Self { expression, slab }
    }

    pub fn eval(&self, data: &mut impl EvalNamespace) -> f32 {
        self.try_eval(data).unwrap()
    }

    fn try_eval(&self, data: &mut impl EvalNamespace) -> Result<f32, fasteval::Error> {
        // let mut ns = &mut StrToF64Namespace::from([("t", 0.5)]);

        Ok(fasteval::eval_compiled_ref!(&self.expression, &self.slab, &mut *data) as f32)
    }
}
