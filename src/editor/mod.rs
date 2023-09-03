use crate::GameState;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText},
    EguiContexts, EguiSettings,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::InEditor), ())
        app.add_systems(
            Update,
            (editor_indicator_ui, toggle_game_mode).run_if(in_state(GameState::InEditor)),
        )
        .add_systems(OnExit(GameState::InEditor), teardown);
    }
}

pub fn editor_indicator_ui(mut contexts: EguiContexts) {
    egui::Area::new("Indicator")
        .anchor(Align2::CENTER_TOP, egui::emath::vec2(10., 5.))
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label(
                    RichText::new("Edit")
                        .font(FontId::proportional(24.))
                        .color(Color32::WHITE),
                );
            });
        });
}

pub fn toggle_game_mode(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_released(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}

pub fn teardown() {}
