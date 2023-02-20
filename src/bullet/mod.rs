mod modifiers;
mod pattern;
mod render;

use std::{path::Path, sync::Arc};

use bevy::{prelude::*, tasks::ComputeTaskPool};
use fasteval::{Slab, StrToF64Namespace};
use itertools::multizip;
use rayon::prelude::*;

use crate::{editor::is_ui_unfocused, player::Player};

use self::{
    modifiers::*,
    pattern::{parse_string, ExpressionSlab, ParsedPattern, ParsedPatterns, PatternLoader},
    render::BulletRenderPlugin,
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BulletModifiersPlugin)
            .init_resource::<ParsedPatterns>()
            .add_plugin(BulletRenderPlugin)
            .add_asset::<ParsedPattern>()
            .init_asset_loader::<PatternLoader>()
            .add_startup_system(load_patterns)
            .init_resource::<BulletContainer>()
            .add_system(BulletContainer::tick_bullets)
            .add_system(move_bullets)
            .add_system(collide_bullets)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused));
        // .add_system(transform_bullets.after(spawn_bullets));
    }
}

#[derive(Component, Clone, Debug)]
pub struct Bullet {
    lifetime: f32,
    position: Vec2,
    rotation: f32,
    angular_velocity: Arc<ExpressionSlab>,
    speed: Arc<ExpressionSlab>,
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            speed: Arc::new(ExpressionSlab::new(
                fasteval::Instruction::IConst(0.),
                Slab::default(),
            )),
            angular_velocity: Arc::new(ExpressionSlab::new(
                fasteval::Instruction::IConst(0.),
                Slab::default(),
            )),
            position: Vec2::default(),
            lifetime: f32::default(),
            rotation: f32::default(),
        }
    }
}

impl Bullet {
    pub fn new(speed: f32) -> Self {
        Self {
            speed: Arc::new(parse_string(speed.to_string().as_str())),
            ..Default::default()
        }
    }
}

fn load_patterns(
    asset_server: Res<AssetServer>,
    // pattern_assets: ResMut<Assets<ParsedPattern>>,
    mut patterns: ResMut<ParsedPatterns>,
) {
    let scripts_iter = asset_server
        .asset_io()
        .read_directory(Path::new("./scripts"))
        .expect("/assets/scripts/ directory doesn't exist.");

    for path in scripts_iter {
        let handle = asset_server.load(path);
        patterns.0.push(handle);
    }
}

fn spawn_bullets(
    mut commands: Commands,
    bullet_container: ResMut<BulletContainer>,
    asset_server: Res<AssetServer>,
    patterns: ResMut<Assets<ParsedPattern>>,
    input: Res<Input<KeyCode>>,
    // player_query: Query<&Transform, With<Player>>,
) {
    if !input.just_pressed(KeyCode::E) {
        return;
    }

    // let player = player_query.single();
    let texture = asset_server.load("SA_bullet.png");

    let handle = asset_server.load("scripts/pattern1.json.pattern");

    let pattern = patterns.get(&handle.clone());
    if let Some(pattern) = pattern {
        pattern.fire(&mut commands, bullet_container, &texture, ());
    }
}

fn collide_bullets(
    player_query: Query<&Transform, (With<Player>, Without<Bullet>)>,
    bullet_query: Query<(Entity, &Bullet)>,
    mut commands: Commands,
) {
    let player_thiccness = 5.;
    let player_thiccness = player_thiccness * player_thiccness;

    for player_tr in player_query.iter() {
        for (entity, bullet) in bullet_query.iter() {
            if player_tr
                .translation
                .distance_squared(bullet.position.extend(0.))
                < player_thiccness
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn move_bullets(mut bullet_query: Query<&mut Bullet>, time: Res<Time>) {
    let mut namespace = StrToF64Namespace::new();
    namespace.insert("t", 0.);

    for mut bullet in bullet_query.iter_mut() {
        let Bullet {
            position,
            rotation,
            speed,
            lifetime,
            angular_velocity,
        } = bullet.as_mut();

        *lifetime += time.delta_seconds();
        namespace.insert("t", *lifetime as f64);
        *rotation += angular_velocity.eval(&mut namespace) as f32 * time.delta_seconds();
        *position +=
            Vec2::from_angle(*rotation) * speed.eval(&mut namespace) as f32 * time.delta_seconds();
    }
}

#[derive(Resource, Clone)]
pub struct BulletContainer {
    lifetimes: Vec<f32>,
    positions: Vec<Vec2>,
    rotations: Vec<f32>,
    speeds: Vec<f32>,
    angulars: Vec<f32>,
}

impl BulletContainer {
    pub fn add(&mut self, lifetime: f32, position: Vec2, rotation: f32, speed: f32, angular: f32) {
        self.lifetimes.push(lifetime);
        self.positions.push(position);
        self.rotations.push(rotation);
        self.speeds.push(speed);
        self.angulars.push(angular);
    }

    fn tick(
        &mut self,
        time: Res<Time>,
        player_query: Query<&Transform, (With<Player>, Without<Bullet>)>,
    ) {
        let time = time.delta_seconds();

        (
            &mut self.lifetimes,
            &mut self.positions,
            &mut self.rotations,
            &self.speeds,
            &self.angulars,
        )
            .into_par_iter()
            .for_each(|(lifetime, position, rotation, speed, angular)| {
                *lifetime += time;
                *position += Vec2::from_angle(*rotation) * *speed * time;
                *rotation += *angular * time;
            });

        let player_thiccness = 5.;
        let player_thiccness = player_thiccness * player_thiccness;

        for player_tr in player_query.iter() {
            let player_pos = player_tr.translation.truncate();
            for bullet_pos in self.positions.iter() {
                if player_pos.distance_squared(*bullet_pos) < player_thiccness {
                    // Remove bullet through collision detection
                }
            }
        }
        /*
        let meshes = ComputeTaskPool::get().scope(|scope| {
            for chunk in slice.chunks(CHUNK_SIZE) {
                scope.spawn(async move { make_mesh(chunk, Vec2::new(8., 14.)) });
            }
        });

        for (lifetime, position, rotation, speed, angular) in multizip((
            &mut self.lifetimes,
            &mut self.positions,
            &mut self.rotations,
            &self.speeds,
            &self.angulars,
        )) {
            *lifetime += time;
            *position += Vec2::from_angle(*rotation) * *speed * time;
            *rotation += *angular * time;
        } */
    }

    fn tick_bullets(
        mut container: ResMut<BulletContainer>,
        time: Res<Time>,
        player_query: Query<&Transform, (With<Player>, Without<Bullet>)>,
    ) {
        container.tick(time, player_query);
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

impl Default for BulletContainer {
    fn default() -> Self {
        const CAPACITY: usize = 100000;

        Self {
            lifetimes: Vec::with_capacity(CAPACITY),
            positions: Vec::with_capacity(CAPACITY),
            rotations: Vec::with_capacity(CAPACITY),
            speeds: Vec::with_capacity(CAPACITY),
            angulars: Vec::with_capacity(CAPACITY),
        }
    }
}
