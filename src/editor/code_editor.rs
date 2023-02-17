// ----------------------------------------------------------------------------

use bevy_egui::egui;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeEditor {
    code: String,
}

pub trait EditorWindow {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}

impl CodeEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { code } = self;

        ui.add(
            egui::TextEdit::multiline(code)
                .code_editor()
                .desired_rows(10).frame(false)
                .desired_width(f32::INFINITY),
        );

        if ui.button("Execute!").clicked() {
            todo!();
        }
    }
}

impl EditorWindow for CodeEditor {
    fn name(&self) -> &'static str {
        "code_editor"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .scroll2([false, true])
            .show(ctx, |ui| self.ui(ui));
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            code: "// Sample Content".into(),
        }
    }
}
