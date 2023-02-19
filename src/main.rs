//! A shader that renders a mesh multiple times in one draw call.

mod bullet;
mod diagnostics;
mod editor;
mod player;

use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};

use bevy_egui::EguiPlugin;
use bevy_mouse_tracking_plugin::{
    prelude::{InsertExt, MousePosPlugin},
    MainCamera,
};
use bullet::BulletPlugin;
use diagnostics::ScreenDiagsPlugin;
use editor::EditorPlugin;
use player::PlayerPlugin;

// mod render;
// use render::BulletMaterialPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 960.,
                height: 540.,
                position: WindowPosition::At(Vec2::new(240., 0.)),
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(ScreenDiagsPlugin)
        .add_plugin(MousePosPlugin)
        .add_plugin(EditorPlugin)
        //.add_plugin(BulletMaterialPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("SA_bullet.png"),
            ..Default::default()
        },
        player::Player,
    ));

    // camera
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .add_mouse_tracking()
        .insert(MainCamera);
}
