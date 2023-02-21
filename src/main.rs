mod bullet;
mod diagnostics;
mod editor;
mod player;

use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};

use bevy_egui::EguiPlugin;
use bullet::BulletPlugin;
use diagnostics::DebugInfoPlugin;
use editor::EditorPlugin;
use player::PlayerPlugin;

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
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_plugin(EguiPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(DebugInfoPlugin)
        .add_plugin(EditorPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("player_temp.png"),
            transform: Transform::from_xyz(0., 100., 0.),
            ..Default::default()
        },
        player::Player,
    ));

    commands.spawn(Camera2dBundle::default());
}
