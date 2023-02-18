use bevy::prelude::*;

use crate::player::Player;

use super::Bullet;

pub struct BulletModifiersPlugin;

impl Plugin for BulletModifiersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(angular_velocity_system)
            .add_system(aimed_system)
            .add_system(delayed_system::<AngularVelocity>)
            .add_system(acceleration_system);
    }
}

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
}

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

#[derive(Component, Clone, Copy)]
pub struct Delayed<T: Component> {
    pub wait: f32,
    pub component: T
}

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

fn acceleration_system(mut bullet_query: Query<(&mut Bullet, &Acceleration)>, time: Res<Time>) {
    let delta_seconds = time.delta_seconds();
    for (mut bullet, acceleration_mod) in bullet_query.iter_mut() {
        bullet.speed += acceleration_mod.amount * delta_seconds;
    }
}

#[derive(Component)]
pub struct Acceleration {
    pub amount: f32,
}
