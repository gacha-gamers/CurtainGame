mod modifiers;
mod pattern;
mod render;

use std::{f32::consts::PI, path::Path, sync::Arc};

use bevy::{ecs::query::QueryItem, prelude::*, render::extract_component::ExtractComponent};
use fasteval::{Slab, StrToF64Namespace};
use itertools::{izip, multizip};

use crate::{editor::is_ui_unfocused, player::Player};

use self::{
    modifiers::*,
    pattern::{parse_string, ExpressionSlab, ParsedPattern, ParsedPatterns, PatternLoader}, render::BulletRenderPlugin,
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
            .add_system(move_bullets)
            .add_system(tick_bullets)
            .add_system(collide_bullets)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused))
            .add_system(transform_bullets.after(spawn_bullets));
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
    mut bullet_container: ResMut<BulletContainer>,
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
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mut commands: Commands,
) {
    let player_thiccness = 5.;
    let player_thiccness = player_thiccness * player_thiccness;

    for player_tr in player_query.iter() {
        for (entity, tr) in bullet_query.iter() {
            if player_tr.translation.distance_squared(tr.translation) < player_thiccness {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn transform_bullets(mut bullet_query: Query<(&mut Transform, &Bullet)>) {
    for (mut tr, bullet) in bullet_query.iter_mut() {
        *tr = calculate_transform(bullet);
    }
}

fn calculate_transform(bullet: &Bullet) -> Transform {
    Transform {
        translation: Vec3 {
            x: bullet.position.x,
            y: bullet.position.y,
            z: 0.,
        },
        rotation: Quat::from_rotation_z(bullet.rotation - PI / 2.),
        ..Default::default()
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

fn tick_bullets(container: ResMut<BulletContainer>, time: Res<Time>) {
    let BulletContainer { mut lifetimes, mut positions, mut rotations, speeds, angulars } = container.to_owned();
    let time = time.delta_seconds();

    for (lifetime, position, rotation, speed, angular) in multizip((
        &mut lifetimes,
        &mut positions,
        &mut rotations,
        &speeds,
        &angulars,
    )) {
        *lifetime += time;
        *position += Vec2::from_angle(*rotation) * *speed * time;
        *rotation += *angular * time;
    }
    /*         for (position, velocity) in self.positions.iter_mut().zip .zip(self.speeds.iter()) {
        *position += *velocity;
    } */
}

impl BulletContainer {
    
    /*
    pub fn add_from_loop<F>(&mut self, count: u32, callback: F)
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let (positions, velocities): (Vec<Vec3>, Vec<Vec3>) = (0..count)
            .map(|i| callback(i as f32 / count as f32))
            .unzip();
        self.positions.extend(positions.iter());
        self.velocities.extend(velocities.iter());
    } */
    /*
    pub fn from_loop<F>(count: u32, callback: F) -> Self
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let mut inst = Self::default();
        inst.add_from_loop(count, callback);
        inst
    } */
}
