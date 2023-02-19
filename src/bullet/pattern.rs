use std::f32::consts::PI;
use std::str::from_utf8;

use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use fasteval::*;
use serde_json::Value;

use super::Bullet;

#[derive(Default)]
pub struct PatternLoader;

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

    fn extensions(&self) -> &[&str] {
        &["json.pattern"]
    }
}

#[derive(Default, Debug, TypeUuid)]
#[uuid = "1ff044c3-1d98-4b22-a7e2-73a41298ff98"]
pub struct ParsedPattern {
    pub operations: Vec<PatternOp>,
    pub instructions: Vec<InstructionI>,
}

impl ParsedPattern {
    fn ring(bullets: Vec<Bullet>, count: u32, radius: f32) -> Vec<Bullet> {
        bullets
            .iter()
            .flat_map(|b| {
                (0..count).map(|i| {
                    let rotation = b.rotation + i as f32 / count as f32 * 2. * PI;
                    Bullet {
                        position: b.position + Vec2::from_angle(rotation) * radius,
                        rotation,
                        ..*b
                    }
                })
            })
            .collect()
    }

    #[allow(dead_code)]
    fn line(bullets: Vec<Bullet>, count: u32, delta_speed: f32) -> Vec<Bullet> {
        bullets
            .iter()
            .flat_map(|b| {
                (0..count).map(|i| Bullet {
                    speed: b.speed + delta_speed * i as f32,
                    ..*b
                })
            })
            .collect()
    }

    fn arc(bullets: Vec<Bullet>, count: u32, angle: f32) -> Vec<Bullet> {
        if count == 1 {
            return bullets;
        }

        let step = angle / (count as f32 - 1.0);
        bullets
            .iter()
            .flat_map(|b| {
                (0..count).map(|i| Bullet {
                    rotation: b.rotation - angle / 2.0 + step * i as f32,
                    ..*b
                })
            })
            .collect()
    }

    pub fn fire(
        &self,
        commands: &mut Commands,
        texture: &Handle<Image>,
        modifiers_bundle: impl Bundle + Copy,
    ) {
        let mut bullets = vec![Bullet::new(60.)];

        for op in self.operations.iter() {
            bullets = match op {
                PatternOp::Ring(count, radius) => {
                    ParsedPattern::ring(bullets, count.eval(EmptyNamespace) as u32, *radius)
                }
                PatternOp::Arc(count, angle) => ParsedPattern::arc(bullets, *count, *angle),
                PatternOp::Bullet(_) => {
                    for bullet_comp in bullets.iter() {
                        commands.spawn((
                            SpriteBundle {
                                texture: texture.clone(),
                                transform: super::calculate_transform(&bullet_comp),
                                ..Default::default()
                            },
                            modifiers_bundle,
                            bullet_comp.clone(),
                        ));
                    }
                    bullets
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ParsedPatterns(pub Vec<Handle<ParsedPattern>>);

#[derive(Debug)]
pub enum PatternOp {
    Ring(Box<ExpressionSlab>, f32),
    Arc(u32, f32 /* fasteval::Expression */),
    Bullet(Bullet),
}

pub fn parse(source: &str) -> ParsedPattern {
    let json: serde_json::error::Result<Value> = serde_json::from_str(source);

    let mut value = &match json {
        Err(error) => {
            panic!("Error while parsing pattern: {error}");
        }
        Ok(t) => t,
    };

    let mut pattern = ParsedPattern::default();

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
                "bullet" => PatternOp::Bullet(Bullet {
                    speed: value["speed"].as_f64().expect("No speed provided.") as f32,
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
    
    let mut slab = Slab::new();
    let expression = fasteval::Parser::new()
        .parse(expression, &mut slab.ps)
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

impl ExpressionSlab {
    fn new(expression: Instruction, slab: Slab) -> Self {
        Self { expression, slab }
    }

    fn eval(&self, data: impl EvalNamespace) -> f64 {
        self.try_eval(data).unwrap()
    }

    fn try_eval(&self, mut data: impl EvalNamespace) -> Result<f64, fasteval::Error> {
        Ok(fasteval::eval_compiled_ref!(
            &self.expression,
            &self.slab,
            &mut data
        ))
    }
}
