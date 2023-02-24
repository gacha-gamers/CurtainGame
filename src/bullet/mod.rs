pub mod pattern;
mod render;

use bevy::prelude::*;
use rayon::prelude::*;

use crate::{
    editor::{is_ui_unfocused, EditorState},
    player::Player,
};

use self::{
    pattern::{Pattern, PatternDatabase, PatternLoader},
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
            .add_startup_system(BulletPool::create_pool_temp)
            .init_asset_loader::<PatternLoader>()
            .add_system(BulletPool::tick_bullets)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused));
    }
}

fn spawn_bullets(
    bullet_pools: Query<&mut BulletPool>,
    patterns: Res<Assets<Pattern>>,
    pattern_db: Res<PatternDatabase>,
    editor_state: Res<EditorState>,
    input: Res<Input<KeyCode>>,
) {
    if !input.just_pressed(KeyCode::E) {
        return;
    }

    let pattern = patterns.get(&pattern_db.get(&editor_state.selected_pattern).unwrap());
    if let Some(pattern) = pattern {
        pattern.fire(bullet_pools);
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

    pool_index: usize,
    pool_count: usize,
}

impl BulletPool {
    const POOL_CAPACITY: usize = 100_000;

    pub fn add(&mut self, lifetime: f32, position: Vec2, rotation: f32, speed: f32, angular: f32) {
        let i = self.pool_index;
        // If the previous bullet at this index was alive, this is a replacement
        let is_replacing = Self::is_alive(self.ages[i]);

        self.ages[i] = 0.;
        self.lifetimes[i] = lifetime;
        self.positions[i] = position;
        self.rotations[i] = rotation;
        self.speeds[i] = speed;
        self.angulars[i] = angular;

        self.pool_index = (self.pool_index + 1) % BulletPool::POOL_CAPACITY;
        self.pool_count += !is_replacing as usize;
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

    fn create_pool_temp(mut commands: Commands) {
        commands.spawn(BulletPool::default());
    }

    fn tick_bullets(
        mut bullet_pools: Query<&mut BulletPool>,
        player_query: Query<&Transform, With<Player>>,
        time: Res<Time>,
    ) {
        for mut bullet_pool in bullet_pools.iter_mut() {
            bullet_pool.tick(&time);
            bullet_pool.check_collisions(player_query.single());
        }
    }

    pub fn len(&self) -> usize {
        self.pool_count
    }

    fn remove(&mut self, i: usize) {
        if !Self::is_alive(self.ages[i]) {
            return;
        }

        self.ages[i] = -1.0;
        self.positions[i].x = f32::MAX;
        self.pool_count -= 1;
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

impl Default for BulletPool {
    fn default() -> Self {
        Self {
            ages: vec![-1.; BulletPool::POOL_CAPACITY],
            lifetimes: vec![0.0; BulletPool::POOL_CAPACITY],
            positions: vec![Vec2::ONE * 100000000.; BulletPool::POOL_CAPACITY],
            rotations: vec![0.; BulletPool::POOL_CAPACITY],
            speeds: vec![0.; BulletPool::POOL_CAPACITY],
            angulars: vec![0.; BulletPool::POOL_CAPACITY],
            pool_index: 0,
            pool_count: 0,
        }
    }
}
