mod code_editor;

use bevy::{prelude::*, utils::HashMap, ecs::schedule::ShouldRun};
use bevy_egui::{
    egui::{self},
    EguiContext,
};

use self::code_editor::{CodeEditor, EditorWindow};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EditorState::default())
            .insert_resource(UIFocus::default())
            .add_system(update_focused)
            .add_system(setup);
    }
}

#[derive(Resource, Default)]
pub struct UIFocus {
    pub has_focus : bool
}

fn update_focused(mut ui : ResMut<UIFocus>, mut ctx: ResMut<EguiContext>) {
    ui.has_focus = ctx.ctx_mut().memory().focus().is_some();
}

pub(crate) fn is_ui_unfocused(ui : Res<UIFocus>) -> ShouldRun {
    ShouldRun::from(!ui.has_focus)
}

#[derive(Resource, Default)]
pub struct EditorState {
    code_editor : CodeEditor,
    window_states : HashMap<String, bool>
}

/*
struct GraphNode {
    id: Id,
} */

fn setup(mut ctx: ResMut<EguiContext> , mut editor_state: ResMut<EditorState>) {
    let ctx = ctx.ctx_mut();
    let EditorState { code_editor, window_states } = editor_state.as_mut();
    
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
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
    });

    let open = window_states.entry(code_editor.name().to_string()).or_insert(false);

    code_editor.show(ctx, open);
    /*
    for (i, node) in editor_state.nodes.iter().enumerate() {
        egui::Window::new("UI")
            .id(node.id)
            .vscroll(true)
            .show(ctx, |ui| {

                // egui::ScrollArea::vertical().show(ui, |ui| ui.label("Function"));
            });
    } */
}