// mod code_editor;
// use self::code_editor::{CodeEditor, EditorWindow};

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_egui::{egui::{self, FontId, RichText}, EguiContext};

use crate::bullet::pattern::PatternDatabase;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIFocus::default())
            .init_resource::<EditorState>()
            .add_system(update_focused)
            .add_system(update);
    }
}

#[derive(Resource, Default)]
pub struct UIFocus {
    pub has_focus: bool,
}

#[derive(Resource, Default)]
pub struct EditorState {
    pub selected_pattern: String,
}

fn update_focused(mut ui: ResMut<UIFocus>, mut ctx: ResMut<EguiContext>) {
    ui.has_focus = ctx.ctx_mut().memory().focus().is_some();
}

pub(crate) fn is_ui_unfocused(ui: Res<UIFocus>) -> ShouldRun {
    ShouldRun::from(!ui.has_focus)
}

fn update(
    mut ctx: ResMut<EguiContext>,
    patterns_db: Res<PatternDatabase>,
    mut editor_state: ResMut<EditorState>,
) {
    egui::SidePanel::right("pattern_list").resizable(false).show(ctx.ctx_mut(), |ui| {
        ui.label("Move with WASD/Arrow keys.");
        ui.separator();
        ui.label("Press E to fire pattern.");
        ui.label("Patterns are obtained from /assets/patterns/");
        ui.label(RichText::new("Available Patterns").font(FontId::proportional(16.0)).strong());
        
        for name in patterns_db.0.keys() {
            if ui.button(name).clicked() || editor_state.selected_pattern.is_empty() {
                editor_state.selected_pattern = name.clone();
            }
        }
        
        ui.label(format!("Selected: {}", editor_state.selected_pattern));
    });

    // let EditorState { code_editor, window_states } = editor_state.as_mut();
    /* egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.button("Edit Code").clicked() {
                window_states.insert(code_editor.name().to_string(), true);
            }
            /*
            egui::menu::menu_button(ui, "Edit UI", |ui| {
                /* if ui.button("Add Node (Ctrl+N)").clicked() {
                    let len = editor_state.nodes.len();
                    editor_state.nodes.push(GraphNode {
                        id: Id::new(len + 0xabea),
                    });
                } */
            }) */
        })
    }); */

    // let open = window_states.entry(code_editor.name().to_string()).or_insert(false);
    // code_editor.show(ctx, open);
}

/*
#[derive(Resource, Default)]
pub struct EditorState {
    code_editor : CodeEditor,
    window_states: HashMap<String, bool>,
} */
