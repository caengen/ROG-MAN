use bevy::ui::ContentSize;
use bevy::{math::vec2, prelude::*};
use bevy_egui::{
    egui::{self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText},
    EguiContexts, EguiSettings,
};
use bevy_turborand::DelegatedRng;
use bevy_turborand::{GlobalRng, RngComponent};
use std::time::Duration;

use crate::GameState;

use super::components::{
    AnimationIndices, AnimationTimer, ExampleGameText, Paused, PausedText, Player, Pos, Vel,
};

pub fn toggle_edit_mode(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_released(KeyCode::Space) {
        next_state.set(GameState::InEditor);
    }
}

pub fn game_keys(
    mut paused: ResMut<Paused>,
    keyboard: Res<Input<KeyCode>>,
    mut player: Query<(
        &Player,
        &mut Transform,
        &mut AnimationIndices,
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
    )>,
) {
}

pub fn game_indicator_ui(mut contexts: EguiContexts) {
    egui::Area::new("Indicator")
        .anchor(Align2::CENTER_TOP, egui::emath::vec2(10., 5.))
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label(
                    RichText::new("Game")
                        .font(FontId::proportional(24.))
                        .color(Color32::WHITE),
                );
            });
        });
}

// pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // Size of the tile map in tiles.
//     let map_size = TilemapSize { x: 32, y: 32 };

//     // To create a map we use the TileStorage component.
//     // This component is a grid of tile entities and is used to help keep track of individual
//     // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
//     // component per layer.
//     let mut tile_storage = TileStorage::empty(map_size);

//     // For the purposes of this example, we consider a tilemap with rectangular tiles.
//     let map_type = TilemapType::Square;

//     let tilemap_entity = commands.spawn_empty().id();

//     // Spawn a 32 by 32 tilemap.
//     // Alternatively, you can use helpers::fill_tilemap.
//     for x in 0..map_size.x {
//         for y in 0..map_size.y {
//             let tile_pos = TilePos { x, y };
//             let tile_entity = commands
//                 .spawn(TileBundle {
//                     position: tile_pos,
//                     tilemap_id: TilemapId(tilemap_entity),
//                     ..Default::default()
//                 })
//                 .id();
//         }
//     }
// }

pub fn teardown(mut commands: Commands, texts: Query<(Entity, With<ExampleGameText>)>) {}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}
