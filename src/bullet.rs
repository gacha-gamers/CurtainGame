use std::f32::consts::PI;

use bevy::{ecs::query::QueryItem, prelude::*};

use crate::player::Player;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_bullets).add_system(spawn_bullets);
    }
}

#[derive(Component)]
pub struct Bullet {
    // velocity: Vec2,
    speed : f32,
    angular_velocity: f32,
}

fn spawn_bullets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
) {
    if !input.just_pressed(KeyCode::E) {
        return;
    }
    //let mut bullet_datas = query.single_mut();

    let texture = asset_server.load("SA_bullet.png");

    let count = 1000;
    commands.spawn_batch(
        (0..count).map(move |i| radial_bullets(i as f32 / count as f32, texture.clone())),
    );

    // let (positions, velocities): (Vec<Vec3>, Vec<Vec3>) = (0..count)
    // .map(|i| callback(i as f32 / count as f32))
    // .unzip();
    // self.positions.extend(positions.iter());
    // self.velocities.extend(velocities.iter());

    // bullet_datas.add_from_loop(10000, |p| {
    // (Vec3::ZERO, Vec2::from_angle(p * 2. * PI).extend(0.) * 0.01)
    // });
    // println!("Bullet count: {}", bullet_datas.positions.len());
}

fn radial_bullets(percent: f32, texture: Handle<Image>) -> impl Bundle {
    let angle = percent * 2. * PI;
    (
        SpriteBundle {
            texture,
            transform: Transform {
                rotation: Quat::from_rotation_z(angle + PI / 2.),
                ..Default::default()
            },
            ..Default::default()
        },
        Bullet {
            // velocity: Vec2::from_angle(angle) * 0.2,
            // angle,
            speed: 0.2,
            angular_velocity: 1.,
        },
    )
}

fn move_bullets(
    player_query: Query<&Transform, (With<Player>, Without<Bullet>)>,
    mut bullet_query: Query<(Entity, &mut Transform, &Bullet)>,
    time: Res<Time>,
    mut commands: Commands
) {
    let player_thiccness = 5.;
    let player_thiccness = player_thiccness * player_thiccness;

    for player_tr in player_query.iter() {
        for (entity, mut tr, bullet) in bullet_query.iter_mut() {
            let rot = tr.rotation.to_euler(EulerRot::XYZ).2;
            tr.translation += (Vec2::from_angle(rot) * bullet.speed).extend(0.);
            tr.rotate_local_z(bullet.angular_velocity * time.delta_seconds());
            // bullet.velocity = Vec2::from_angle(
            //     bullet.velocity.y.atan2(bullet.velocity.x)
            //         + bullet.angular_velocity * time.delta_seconds(),
            // );

            if player_tr.translation.distance_squared(tr.translation) < player_thiccness {
                commands.entity(entity).despawn();
            }
        }
    }
}

/*
impl BulletContainer {
    pub fn process_velocities(&mut self) {
        for (position, velocity) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *position += *velocity;
        }
    }

    pub fn add_from_loop<F>(&mut self, count: u32, callback: F)
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let (positions, velocities): (Vec<Vec3>, Vec<Vec3>) = (0..count)
            .map(|i| callback(i as f32 / count as f32))
            .unzip();
        self.positions.extend(positions.iter());
        self.velocities.extend(velocities.iter());
    }

    pub fn from_loop<F>(count: u32, callback: F) -> Self
    where
        F: Fn(f32) -> (Vec3, Vec3),
    {
        let mut inst = Self::default();
        inst.add_from_loop(count, callback);
        inst
    }
}

impl ExtractComponent for BulletContainer {
    type Query = &'static BulletContainer;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Self {
        item.clone()
    }
}
 */
