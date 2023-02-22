use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::FixedTimestep,
};
use bevy_egui::{egui, EguiContext};

use crate::bullet::BulletPool;

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
pub struct Framerate {
    average: f64,
    min: f64,
}

fn update(
    bullet_container: ResMut<BulletPool>,
    framerate: Res<Framerate>,
    mut ctx: ResMut<EguiContext>,
) {
    egui::TopBottomPanel::top("debug")
        .frame(egui::Frame::none())
        .show_separator_line(false)
        .show(ctx.ctx_mut(), |ui| {
            ui.label(format!(
                "FPS: {:.0} (min {:.0})",
                framerate.average, framerate.min
            ));
            ui.label(format!("Bullets: {:.0}", bullet_container.len()));
        });
}

fn extract_fps(mut framerate: ResMut<Framerate>, diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| {
            Some((
                fps.average(),
                fps.measurements()
                    .map(|m| m.value)
                    .min_by(|m1, m2| m1.total_cmp(&m2))
            ))
        })
    {
        framerate.average = fps.0.unwrap();
        framerate.min = fps.1.unwrap();
    }
}
