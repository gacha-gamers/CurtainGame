mod modifiers;

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{editor::is_ui_unfocused, player::Player};

use self::modifiers::{AngularVelocity, BulletModifiersPlugin};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BulletModifiersPlugin)
            .add_system(move_bullets)
            .add_system(collide_bullets)
            .add_system(transform_bullets)
            .add_system(spawn_bullets.with_run_criteria(is_ui_unfocused));
    }
}

#[derive(Component, Default)]
pub struct Bullet {
    position: Vec2,
    rotation: f32,
    speed: f32,
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

    let texture = asset_server.load("SA_bullet.png");
/* 
    let pattern = Pattern::new()
        .ring(8, 1.)
        .fire(commands, radial_bullets(0., texture.clone(), ()));
 */
    
    let count = 10 - 000;
    commands.spawn_batch((0..count).map(move |i| {
        radial_bullets(
            i as f32 / count as f32,
            texture.clone(),
            AngularVelocity { amount: 0.5 },
        )
    }));
}
/* 
struct Pattern {
    bullets: Vec<Bullet>,
}

impl Pattern {
    fn new() -> Self {
        Self {
            bullets: vec![Bullet {
                speed: 1.,
                ..Default::default()
            }],
        }
    }

    fn ring(&mut self, count: u32, radius: f32) -> &mut Self {
        self.bullets = self
            .bullets
            .iter_mut()
            .flat_map(|b| {
                (0..count).map(|i| Bullet {
                    rotation: i as f32 / count as f32 * 2. * PI,
                    ..*b
                })
            })
            .collect();
        self
    }

    fn fire<T: Bundle>(&self, mut commands: Commands, bundle: T) {
        commands.spawn_batch(self.bullets.iter().map(|b| (bundle, *b)));
    }
} */

fn radial_bullets(percent: f32, texture: Handle<Image>, bundle: impl Bundle) -> impl Bundle {
    let angle = percent * 2. * PI;
    (
        SpriteBundle {
            texture,
            ..Default::default()
        },
        Bullet {
            // velocity: Vec2::from_angle(angle) * 0.2,
            position: Vec2::ZERO,
            rotation: angle,
            speed: 0.2,
            angular_velocity: 0.,
        },
        bundle,
    )
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
        *tr = Transform {
            translation: Vec3 {
                x: bullet.position.x,
                y: bullet.position.y,
                z: 0.,
            },
            rotation: Quat::from_rotation_z(bullet.rotation - PI / 2.),
            ..Default::default()
        };
    }
}

fn move_bullets(mut bullet_query: Query<&mut Bullet>, time: Res<Time>) {
    for mut bullet in bullet_query.iter_mut() {
        let rotation = bullet.rotation + bullet.angular_velocity * time.delta_seconds();
        let speed = bullet.speed;

        bullet.rotation = rotation;
        bullet.position += Vec2::from_angle(rotation) * speed;
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
