use std::f32::consts::PI;

use bevy::{ecs::query::QueryItem, prelude::*, render::extract_component::ExtractComponent};
use bytemuck::{Pod, Zeroable};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_bullets).add_system(spawn_bullets);
    }
}

fn spawn_bullets(mut query: Query<&mut BulletContainer>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::E) {
        let mut bullet_datas = query.single_mut();
        bullet_datas.add_from_loop(10000, |p| {
            (Vec3::ZERO, Vec2::from_angle(p * 2. * PI).extend(0.) * 0.01)
        });
        println!("Bullet count: {}", bullet_datas.positions.len());
    }
}

fn move_bullets(mut query: Query<&mut BulletContainer>) {
    query.for_each_mut(|mut b| b.process_velocities());
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BulletModel {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}

#[derive(Component, Default, Clone)]
pub struct BulletContainer {
    pub positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
}

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
