use bevy::prelude::*;

use super::Bullet;

pub struct BulletModifiersPlugin;

impl Plugin for BulletModifiersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(angular_velocity_system)
            .add_system(acceleration_system);
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

#[derive(Component)]
pub struct AngularVelocity {
    pub amount: f32,
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
