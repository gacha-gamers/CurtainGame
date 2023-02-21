use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::FixedTimestep,
};
use bevy_egui::{egui, EguiContext};

use crate::bullet::BulletContainer;

/// Originally from the ScreenDiags crate: https://github.com/jomala/bevy_screen_diags
pub struct DebugInfoPlugin;

impl Plugin for DebugInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<Framerate>()
            .add_system_set(
                SystemSet::new()
                    .with_system(extract_fps)
                    .with_run_criteria(FixedTimestep::steps_per_second(2.)),
            )
            .add_system_to_stage(CoreStage::PostUpdate, update);
    }
}

#[derive(Resource, Default)]
pub struct Framerate(f64);

fn update(
    bullet_container: ResMut<BulletContainer>,
    framerate: Res<Framerate>,
    mut ctx: ResMut<EguiContext>,
) {
    egui::TopBottomPanel::top("debug").frame(egui::Frame::none()).show_separator_line(false).show(ctx.ctx_mut(), |ui| {
        ui.label(format!("FPS: {:.0}", framerate.0));
        ui.label(format!("Bullets: {:.0}", bullet_container.len()));
    });
}

fn extract_fps(mut framerate: ResMut<Framerate>, diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        framerate.0 = fps;
    }
}
