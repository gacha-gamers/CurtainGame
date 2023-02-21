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
    render::BulletRenderPlugin,
};

const PLAYER_RADIUS: f32 = 5.;
const PLAYER_RADIUS_SQR: f32 = PLAYER_RADIUS * PLAYER_RADIUS;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatternDatabase>()
            .add_plugin(BulletRenderPlugin)
            .add_asset::<Pattern>()
            .add_startup_system(PatternLoader::init_database)
            .init_asset_loader::<PatternLoader>()
            .init_resource::<BulletContainer>()
            .add_system(BulletContainer::tick_bullets)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused));
    }
}

fn spawn_bullets(
    bullet_container: ResMut<BulletContainer>,
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
        pattern.fire(bullet_container);
    }
}

#[derive(Resource, Clone)]
pub struct BulletContainer {
    ages: Vec<f32>,
    lifetimes: Vec<f32>,
    positions: Vec<Vec2>,
    rotations: Vec<f32>,
    speeds: Vec<f32>,
    angulars: Vec<f32>,
    bullet_index: usize,
    bullet_count: usize,
}

impl BulletContainer {
    const CAPACITY: usize = 60000;

    pub fn add(&mut self, lifetime: f32, position: Vec2, rotation: f32, speed: f32, angular: f32) {
        let i = self.bullet_index;
        // Temporary, quite dirty approach for detecting if a bullet exists
        let is_replacing = self.positions[i].x < 100000.0;

        self.ages[i] = 0.;
        self.lifetimes[i] = lifetime;
        self.positions[i] = position;
        self.rotations[i] = rotation;
        self.speeds[i] = speed;
        self.angulars[i] = angular;

        self.bullet_index = (self.bullet_index + 1) % BulletContainer::CAPACITY;
        self.bullet_count += !is_replacing as usize;
    }

    fn tick(&mut self, time: Res<Time>) {
        let time = time.delta_seconds();

        let mut to_remove: Vec<usize> = (
            &mut self.ages,
            &mut self.lifetimes,
            &mut self.positions,
            &mut self.rotations,
            &self.speeds,
            &self.angulars,
        )
            .into_par_iter()
            .enumerate()
            .filter_map(|(i, (age, lifetime, position, rotation, speed, angular))| {
                *position += Vec2::from_angle(*rotation) * *speed * time;
                *rotation += *angular * time;
                
                // Dirty check if bullet is not dead
                if position.x < 100000. {
                    *age += time;
                    if *age > *lifetime {
                        return Some(i);
                    }
                }

                None
            }).collect();

        to_remove.sort();
        to_remove.reverse();
        for r in to_remove {
            self.remove(r);
        }
    }

    fn check_collisions(&mut self, player_tr: &Transform) {
        let player_pos = player_tr.translation.truncate();

        let mut to_remove = vec![];
        for (i, bullet_pos) in self.positions.iter().enumerate() {
            if player_pos.distance_squared(*bullet_pos) < PLAYER_RADIUS_SQR {
                to_remove.push(i);
            }
        }

        // Remove bullets marked to remove in the correct order
        to_remove.sort();
        to_remove.reverse();
        for r in to_remove {
            self.remove(r);
        }
    }

    fn tick_bullets(
        mut container: ResMut<BulletContainer>,
        player_query: Query<&Transform, With<Player>>,
        time: Res<Time>,
    ) {
        container.tick(time);
        container.check_collisions(player_query.single());
    }

    pub fn len(&self) -> usize {
        self.bullet_count
    }

    fn remove(&mut self, i: usize) {
        // Temporary, quite dirty approach for deleting bullets
        self.positions[i] = Vec2::ONE * 100000000.;
        self.speeds[i] = 0.0;
        self.bullet_count -= 1;
    }
}

impl Default for BulletContainer {
    fn default() -> Self {
        Self {
            lifetimes: vec![0.0; BulletContainer::CAPACITY],
            positions: vec![Vec2::ONE * 100000000.; BulletContainer::CAPACITY],
            rotations: vec![0.; BulletContainer::CAPACITY],
            speeds: vec![0.; BulletContainer::CAPACITY],
            angulars: vec![0.; BulletContainer::CAPACITY],
            ages: vec![0.; BulletContainer::CAPACITY],
            bullet_index: 0,
            bullet_count: 0,
        }
    }
}
