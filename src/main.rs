//! A shader that renders a mesh multiple times in one draw call.

mod bullet;
mod player;
mod diagnostics;

use bevy::{prelude::*, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};

use bullet::{BulletPlugin};
use diagnostics::ScreenDiagsPlugin;
use player::PlayerPlugin;

// mod render;
// use render::BulletMaterialPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(ScreenDiagsPlugin)
        //If a UI camera is already in your game remove the next line
        // .add_startup_system(|mut commands: Commands| {commands.spawn(UiCameraConfig);})
        //.add_plugin(BulletMaterialPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>) {
    /* commands.spawn((
        meshes.add(Mesh::from(shape::Quad {
            size: Vec2::splat(0.1),
            flip: false,
        })),
        SpatialBundle::VISIBLE_IDENTITY,
        BulletContainer::from_loop(15, |p| {
            (Vec3::ZERO, Vec2::from_angle(p * 2. * PI).extend(0.) * 0.05)
        }),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    ));
    (0..15)
        .map(|i| BulletData {
            position: Vec3::ZERO,
            scale: 1.0,
            color: Color::hsla(i as f32 / 15. * 360., 0.5, 1., 1.).as_rgba_f32(),
        })
        .collect()
 */

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("SA_bullet.png"),
            ..Default::default()
        },
        player::Player,
    ));
    
    // camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
