use crate::editor::is_ui_unfocused;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_controls.with_run_criteria(is_ui_unfocused));
    }
}

fn player_controls(mut query: Query<&mut Transform, With<Player>>, key_input: Res<Input<KeyCode>>) {
    for mut tr in query.iter_mut() {
        let h_movement = key_input.any_pressed([KeyCode::D, KeyCode::Right]) as i32
            - key_input.any_pressed([KeyCode::A, KeyCode::Left]) as i32;
        let v_movement = key_input.any_pressed([KeyCode::W, KeyCode::Up]) as i32
            - key_input.any_pressed([KeyCode::S, KeyCode::Down]) as i32;

        let movement = Vec2 {
            x: h_movement as f32,
            y: v_movement as f32,
        }
        .normalize_or_zero();

        tr.translation += movement.extend(0.);
    }
}
