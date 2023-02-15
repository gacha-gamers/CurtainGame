use std::f32::consts::PI;

use bevy::prelude::*;

#[derive(Component, Default)]
struct Bullet {
    speed: f32,
    direction: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(initialize_game)
        .add_system(process_bullets)
        .run();
}

fn initialize_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    spawn_bullets(commands, asset_server);
}

fn process_bullets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Transform, &Bullet)>,
    input: Res<Input<KeyCode>>,
) {
    for (mut tr, bullet) in query.iter_mut() {
        tr.rotation = Quat::from_rotation_z(bullet.direction + PI / 2.0);
        tr.translation += (Vec2::from_angle(bullet.direction) * bullet.speed).extend(0.0);
    }

    if input.just_pressed(KeyCode::E) {
        spawn_bullets(commands, asset_server);
    }
}

fn spawn_bullets(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in 0..10000 {
        for (position, velocity) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *position += *velocity * dt;
        }
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("SA_bullet.png"),
                ..Default::default()
            },
            Bullet {
                speed: 1.0,
                direction: i as f32 / 100.0 * 2.0 * PI,
                ..Default::default()
            },
        ));
    }
}
