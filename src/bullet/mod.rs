pub mod pattern;
mod render;

use std::{collections::BTreeMap, ops::Range, sync::Arc};

use bevy::prelude::*;
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
    ages: Vec<f32>,
    lifetimes: Vec<f32>,
    positions: Vec<Vec2>,
    rotations: Vec<f32>,
    speeds: Vec<f32>,
    angulars: Vec<f32>,

    modifiers: Vec<BulletModifier>,
    handle: Handle<Image>,
    index: usize,
    count: usize,
    capacity: usize,
}

impl BulletPool {
    fn new(capacity: usize, handle: Handle<Image>) -> Self {
        Self {
            ages: vec![-1.; capacity],
            lifetimes: vec![0.0; capacity],
            positions: vec![Vec2::ONE * 100000000.; capacity],
            rotations: vec![0.; capacity],
            speeds: vec![0.; capacity],
            angulars: vec![0.; capacity],

            modifiers: Default::default(),
            index: 0,
            count: 0,
            capacity,
            handle,
        }
    }

    pub fn add_modifier(&mut self, modifier: BulletModifier) {
        self.modifiers.push(modifier);
    }

    pub fn add(&mut self, lifetime: f32, position: Vec2, rotation: f32, speed: f32, angular: f32) {
        let i = self.index;
        // If the previous bullet at this index was alive, this is a replacement
        let is_replacing = Self::is_alive(self.ages[i]);

        self.ages[i] = 0.;
        self.lifetimes[i] = lifetime;
        self.positions[i] = position;
        self.rotations[i] = rotation;
        self.speeds[i] = speed;
        self.angulars[i] = angular;

        self.index = (self.index + 1) % self.capacity;
        self.count += !is_replacing as usize;
    }

    fn tick(&mut self, time: &Res<Time>) {
        let delta_time = time.delta_seconds();

        let to_remove: Vec<usize> = (
            &mut self.ages,
            &mut self.lifetimes,
            &mut self.positions,
            &mut self.rotations,
            &self.speeds,
            &self.angulars,
        )
            .into_par_iter()
            .enumerate()
            .filter_map(
                |(index, (age, lifetime, position, rotation, speed, angular))| {
                    if Self::is_alive(*age) {
                        *position += Vec2::from_angle(*rotation) * *speed * delta_time;
                        *rotation += *angular * delta_time;

                        *age += delta_time;
                        if *age > *lifetime {
                            // Return this bullet's index so it can be removed
                            return Some(index);
                        }
                    }

                    None
                },
            )
            .collect();

        self.remove_many(to_remove);
    }

    fn check_collisions(&mut self, player_tr: &Transform) {
        let player_pos = player_tr.translation.truncate();

        let to_remove: Vec<usize> = self
            .positions
            .iter()
            .enumerate()
            .filter_map(|(index, bullet_pos)| {
                // If the bullet is within reach of the player, return its index so it can be removed
                (player_pos.distance_squared(*bullet_pos) < PLAYER_RADIUS_SQR).then_some(index)
            })
            .collect();

        self.remove_many(to_remove);
    }

    fn tick_modifiers(&mut self) {
        if self.ages.len() == 0 {
            return;
        }

        let mut params = StrToF64Namespace::new();
        params.insert("t", self.ages[0] as f64);

        for modifier in self.modifiers.iter() {
            let value =  modifier.expression.eval(&mut params);
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
        for (entity, _) in pool_query.iter().filter(|(_, pool)| pool.count == 0) {
            commands.entity(entity).despawn();
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    fn remove(&mut self, i: usize) {
        if !Self::is_alive(self.ages[i]) {
            return;
        }

        self.ages[i] = -1.0;
        self.positions[i].x = f32::MAX;
        self.count -= 1;
    }

    fn remove_many(&mut self, mut indices: Vec<usize>) {
        indices.sort();
        indices.reverse();
        for r in indices {
            self.remove(r);
        }
    }

    /// Checks if a bullet is alive based on its age
    fn is_alive(age: f32) -> bool {
        age.is_sign_positive()
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
