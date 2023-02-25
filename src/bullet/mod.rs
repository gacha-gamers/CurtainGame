pub mod pattern;
mod render;

use std::{collections::BTreeMap, ops::Range, sync::Arc};

use bevy::{math::Vec3A, prelude::*};
use fasteval::StrToF64Namespace;
use rayon::prelude::*;

use crate::{
    editor::{is_ui_unfocused, EditorState},
    player::Player,
};

use self::{
    pattern::{ExpressionSlab, Pattern, PatternDatabase, PatternLoader},
    render::BulletPipelinePlugin,
};

const PLAYER_RADIUS: f32 = 5.;
const PLAYER_RADIUS_SQR: f32 = PLAYER_RADIUS * PLAYER_RADIUS;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatternDatabase>()
            // .add_plugin(BulletRenderPlugin)
            .add_plugin(BulletPipelinePlugin)
            .add_asset::<Pattern>()
            .add_startup_system(PatternLoader::init_database)
            // .add_startup_system(BulletPool::create_pool)
            .init_asset_loader::<PatternLoader>()
            .init_resource::<BulletPools>()
            .add_system(BulletPool::tick_pools)
            .add_system(BulletPool::free_pools)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused));
    }
}

fn spawn_bullets(
    commands: Commands,
    asset_server: Res<AssetServer>,
    patterns: Res<Assets<Pattern>>,
    // query_pool: Query<&mut BulletPool>,
    // bullet_pools: ResMut<BulletPools>,
    pattern_db: Res<PatternDatabase>,
    editor_state: Res<EditorState>,
    input: Res<Input<KeyCode>>,
) {
    if !input.just_pressed(KeyCode::E) {
        return;
    }

    let pattern = patterns.get(&pattern_db.get(&editor_state.selected_pattern).unwrap());
    if let Some(pattern) = pattern {
        pattern.fire(commands, asset_server /* , query_pool, bullet_pools */);
    }
}

#[derive(Component, Clone)]
pub struct BulletPool {
    states: Vec<Vec3A>,
    speeds: Vec<f32>,
    angulars: Vec<f32>,

    modifiers: Vec<BulletModifier>,
    handle: Handle<Image>,
    index: usize,
    capacity: usize,
    lifetime: f32,
    age: f32,
}

impl BulletPool {
    fn new(capacity: usize, lifetime: f32, handle: Handle<Image>) -> Self {
        Self {
            states: vec![Vec3A::new(f32::MAX, 0., 0.); capacity],
            // velocities: vec![Vec4::ZERO; capacity],
            speeds: vec![0.; capacity],
            angulars: vec![0.; capacity],

            modifiers: Default::default(),
            index: 0,
            age: 0.0,
            capacity,
            handle,
            lifetime,
        }
    }

    pub fn add_modifier(&mut self, modifier: BulletModifier) {
        self.modifiers.push(modifier);
    }

    pub fn add(&mut self, position: Vec2, rotation: f32, speed: f32, angular: f32) {
        let i = self.index;

        self.states[i] = Vec3A::new(position.x, position.y, rotation);
        self.speeds[i] = speed;
        self.angulars[i] = angular;

        self.index = (self.index + 1) % self.capacity;
    }

    fn tick(&mut self, time: &Res<Time>) {
        let delta_time = time.delta_seconds();
        self.age += delta_time;

        (&mut self.states, &self.speeds, &self.angulars)
            .into_par_iter()
            .for_each(|(state, speed, angular)| {
                *state +=
                    Vec3A::from((Vec2::from_angle(state.z) * *speed).extend(*angular) * delta_time);
            })
    }

    fn check_collisions(&mut self, player_tr: &Transform) {
        let player_pos = Vec3A::from(player_tr.translation);

        for (i, bullet_pos) in self.states.iter().enumerate() {
            if player_pos.distance_squared(*bullet_pos * Vec3A::new(1., 1., 0.)) < PLAYER_RADIUS_SQR
            {
                self.remove(i);
                break;
            }
        }
    }

    fn tick_modifiers(&mut self) {
        let mut params = StrToF64Namespace::new();
        params.insert("t", self.age as f64);

        for modifier in self.modifiers.iter() {
            let value = modifier.expression.eval(&mut params);
            for i in modifier.range.clone() {
                match modifier.property {
                    ModifierProperty::Speed => self.speeds[i] = value,
                    ModifierProperty::Angular => self.angulars[i] = value,
                }
            }
        }
    }

    fn tick_pools(
        mut pool_query: Query<&mut BulletPool>,
        player_query: Query<&Transform, With<Player>>,
        time: Res<Time>,
    ) {
        pool_query.par_for_each_mut(4, |mut bullet_pool| {
            bullet_pool.tick(&time);
            bullet_pool.check_collisions(player_query.single());

            bullet_pool.tick_modifiers();
        });
    }

    fn free_pools(mut commands: Commands, pool_query: Query<(Entity, &BulletPool)>) {
        for (entity, _) in pool_query
            .iter()
            .filter(|(_, pool)| pool.age >= pool.lifetime)
        {
            commands.entity(entity).despawn();
        }
    }

    pub fn len(&self) -> usize {
        self.capacity
    }

    fn remove(&mut self, i: usize) {
        self.states[i].x = f32::MAX;
    }
}

#[derive(Resource, Default)]
pub struct BulletPools(BTreeMap<String, Entity>);

#[derive(Clone)]
pub struct BulletModifier {
    range: Range<usize>,
    expression: Arc<ExpressionSlab>,
    property: ModifierProperty,
}

#[derive(Clone)]
enum ModifierProperty {
    Speed,
    Angular,
}
