use std::sync::Arc;

use bevy::prelude::*;
use fasteval::StrToF64Namespace;

use crate::player::Player;

use super::{pattern::ExpressionSlab, Bullet};

pub struct BulletModifiersPlugin;

impl Plugin for BulletModifiersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(angular_velocity_system)
            .add_system(update_bullets)
            .add_system(aimed_system);
        // .add_system(delayed_system::<AngularVelocity>)
        // .add_system(acceleration_system);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ModifierTarget {
    Speed,
    AngularVelocity,
}

#[derive(Component, Debug)]
pub struct BulletModifier {
    pub expression_slab: Arc<ExpressionSlab>,
    pub bullets: Vec<Entity>,
    pub target: ModifierTarget,
}

impl BulletModifier {
    pub fn clone_with_bullets(&self, bullets: Vec<Entity>) -> Self {
        Self {
            expression_slab: self.expression_slab.clone(),
            target: self.target,
            bullets,
        }
    }
}

fn update_bullets(
    modifiers: Query<&BulletModifier>,
    mut bullet_query: Query<&mut Bullet>,
    // time: Res<Time>,
) {
    let mut namespace = StrToF64Namespace::new();

    for modifier in modifiers.iter() {
        let mut iter_many_mut = bullet_query.iter_many_mut(&modifier.bullets);
        // let mut iter_many_mut = bullet_query.iter_mut();
        // let a = bullet_query.get_many_mut(modifier.bullets.arr .iter());
        
        while let Some(mut bullet) = iter_many_mut.fetch_next() {
            let Bullet {
                lifetime,
                speed,
                angular_velocity,
                ..
            } = bullet.as_mut();
            namespace.insert("t", *lifetime as f64);

            let store_to = match modifier.target {
                ModifierTarget::Speed => speed,
                ModifierTarget::AngularVelocity => angular_velocity,
            };

            *store_to = modifier.expression_slab.eval(&mut namespace) as f32;
        }
    }
}
/*
fn delayed_system<T: Component>(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Delayed<T>)>,
    time: Res<Time>,
) {
    let delta_seconds = time.delta_seconds();
    for (entity, mut delayed) in bullet_query.iter_mut() {
        delayed.wait -= delta_seconds;
        if delayed.wait < 0. {
            //commands.entity(entity).insert(delayed.component);
        }
    }
} */

fn angular_velocity_system(
    mut bullet_query: Query<(&mut Bullet, &AngularVelocity)>,
    time: Res<Time>,
) {
    let delta_seconds = time.delta_seconds();
    for (mut bullet, angular_velocity_mod) in bullet_query.iter_mut() {
        bullet.rotation += angular_velocity_mod.amount * delta_seconds;
    }
}

fn aimed_system(
    mut bullet_query: Query<(&mut Bullet, &Transform), With<Aimed>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player = player_query.single();

    for (mut bullet, tr) in bullet_query.iter_mut() {
        let offset = player.translation - tr.translation;
        bullet.rotation = offset.y.atan2(offset.x);
    }
}
/*
#[derive(Component, Clone, Copy)]
pub struct Delayed<T: Component> {
    pub wait: f32,
    pub component: T
} */

#[derive(Component, Clone, Copy)]
pub struct Aimed;

#[derive(Component, Clone, Copy)]
pub struct AngularVelocity {
    pub amount: f32,
}

impl AngularVelocity {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}
/*
fn acceleration_system(mut bullet_query: Query<(&mut Bullet, &Acceleration)>, time: Res<Time>) {
    let delta_seconds = time.delta_seconds();
    for (mut bullet, acceleration_mod) in bullet_query.iter_mut() {
        bullet.speed += acceleration_mod.amount * delta_seconds;
    }
}
 */
#[derive(Component)]
pub struct Acceleration {
    pub amount: f32,
}
