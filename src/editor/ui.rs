use bevy::prelude::ResMut;
use bevy_egui::{
    egui::{self, style, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText},
    EguiContexts, EguiSettings,
};

use super::components::{RogBrush, TileMaterial};

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

pub fn brush_mode_ui(mut contexts: EguiContexts, mut brush: ResMut<RogBrush>) {
    egui::Area::new("Brush")
        .anchor(Align2::CENTER_TOP, egui::emath::vec2(0., -50.))
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                if ui.button("Wall").clicked() {
                    // todo: SetMaterial?
                    brush.material = TileMaterial::Wall;
                }
                // todo: SetMaterial?
                if ui.button("Eraser").clicked() {
                    brush.material = TileMaterial::Floor;
                }
            });
        });
}

pub fn brush_panel_ui(mut contexts: EguiContexts, mut brush: ResMut<RogBrush>) {
    egui::Area::new("Brush Panel")
        .anchor(Align2::RIGHT_TOP, egui::emath::vec2(0., 100.))
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                if ui.button("Wall").clicked() {
                    // todo: SetMaterial?
                    brush.material = TileMaterial::Wall;
                }
                // todo: SetMaterial?
                if ui.button("Eraser").clicked() {
                    brush.material = TileMaterial::Floor;
                }
            })
        });
}
