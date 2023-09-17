use crate::{game::prelude::MainCamera, GameState, ImageAssets};
use bevy::{math::Vec4Swizzles, prelude::*, reflect::Tuple, transform::commands};
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::{self, Neighbors, SquareDirection},
    prelude::*,
};
use bevy_egui::{
    egui::{self, style, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText},
    EguiContexts, EguiSettings,
};

pub struct EditorPlugin;

mod components;
use components::*;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InEditor), setup_blank_level)
            .add_event::<AddEditActionEvent>()
            .add_event::<UndoEditActionEvent>()
            .add_event::<RedoEditActionEvent>()
            .add_systems(
                Update,
                (
                    key_input,
                    toggle_game_mode,
                    tile_click,
                    add_edit_actions,
                    undo_edit_action,
                    redo_edit_action,
                    update_board,
                    editor_indicator_ui,
                    brush_mode_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::InEditor)),
            )
            .add_systems(OnExit(GameState::InEditor), teardown)
            .insert_resource(RogBrush::default())
            .insert_resource(ActionStack::default());
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

pub fn key_input(
    keyboard: Res<Input<KeyCode>>,
    mut undo_edit_action: EventWriter<UndoEditActionEvent>,
    mut redo_edit_action: EventWriter<RedoEditActionEvent>,
) {
    if keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_released(KeyCode::Z) {
                redo_edit_action.send(RedoEditActionEvent);
            }
        } else {
            if keyboard.just_released(KeyCode::Z) {
                undo_edit_action.send(UndoEditActionEvent);
            }
        }
    }
}

pub fn toggle_game_mode(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_released(KeyCode::Space) {
        // next_state.set(GameState::InGame);
    }
}

fn material_to_index(material: &TileMaterial) -> u32 {
    match material {
        TileMaterial::Wall => 29,
        TileMaterial::Floor => 32,
    }
}

fn index_to_material(index: u32) -> TileMaterial {
    match index {
        29 => TileMaterial::Wall,
        32 => TileMaterial::Floor,
        _ => TileMaterial::Wall,
    }
}

fn commit_action(
    commands: &mut Commands,
    storage: &TileStorage,
    tile_query: &Query<&TileMaterial>,
    action: &EditAction,
) -> EditAction {
    let undo: EditAction = match action {
        EditAction::PlaceTile {
            material, tile_pos, ..
        } => {
            storage.get(&tile_pos).map_or(action.clone(), |entity| {
                match tile_query.get(entity) {
                    Ok(current_material) => {
                        let undo = EditAction::PlaceTile {
                            tile_pos: *tile_pos,
                            material: current_material.clone(),
                            size: 1,
                        };

                        // Mutation
                        // index.0 = material_to_index(&material);
                        // We insert a material component on the entity. It will be consumed by the draw system
                        // and the texture index will be updated.
                        commands.entity(entity).insert(material.clone());
                        undo
                    }
                    Err(err) => {
                        println!("Entity does not exist: {}", err);

                        action.clone()
                    }
                }
            })
        }
        _ => action.clone(),
    };

    undo
}

fn update_board(
    tilemap_storage: Query<(&TileStorage, &TilemapSize)>,
    mut tiles: Query<(Entity, &mut TileTextureIndex, &TilePos, &TileMaterial)>,
) {
    let (map_storage, map_size) = tilemap_storage.single();

    let mut tiles_to_update: Vec<TileMapIndex> = Vec::new();

    for (_, _, tile_pos, material) in tiles.iter() {
        match material {
            TileMaterial::Wall => {
                let neighbor_positions =
                    Neighbors::get_square_neighboring_positions(&tile_pos, &map_size, false);
                let neighbor_entities = neighbor_positions.entities(&map_storage);
                let dirs = &[
                    SquareDirection::North,
                    SquareDirection::East,
                    SquareDirection::South,
                    SquareDirection::West,
                ];
                let dirs = dirs
                    .iter()
                    .map(|dir| {
                        if let Some(entity) = neighbor_entities.get(dir.clone()) {
                            if let Ok((_, _, _, material)) = tiles.get(*entity) {
                                return match material {
                                    TileMaterial::Wall => true,
                                    TileMaterial::Floor => false,
                                };
                            }
                        }

                        return false;
                    })
                    .collect::<Vec<_>>();

                let tile_map_index = match dirs[..] {
                    [false, true, true, true] => TileMapIndex::WallOXXX,
                    [true, false, true, true] => TileMapIndex::WallXOXX,
                    [false, true, true, false] => TileMapIndex::WallOXXO,
                    [false, true, false, true] => TileMapIndex::WallOXOX,
                    [false, false, true, true] => TileMapIndex::WallOOXX,
                    [true, true, true, true] => TileMapIndex::WallXXXX,
                    [true, true, true, false] => TileMapIndex::WallXXXO,
                    [true, true, false, true] => TileMapIndex::WallXXOX,
                    [false, false, false, true] => TileMapIndex::WallOOOX,
                    [true, false, true, false] => TileMapIndex::WallXOXO,
                    [false, false, false, false] => TileMapIndex::WallOXOX, // no connections
                    [true, false, false, false] => TileMapIndex::WallXOOO,
                    [false, false, true, false] => TileMapIndex::WallOOXO,
                    [true, true, false, false] => TileMapIndex::WallXXOO,
                    [false, true, false, false] => TileMapIndex::WallOXOO,
                    [true, false, false, true] => TileMapIndex::WallXOOX,
                    _ => TileMapIndex::Floor,
                };

                tiles_to_update.push(tile_map_index.clone());
            }
            TileMaterial::Floor => {
                tiles_to_update.push(TileMapIndex::Floor);
            }
        }
    }

    tiles
        .iter_mut()
        .enumerate()
        .for_each(|(idx, (_, mut index, ..))| {
            index.0 = tiles_to_update[idx].clone() as u32;
        });
}

pub fn redo_edit_action(
    mut commands: Commands,
    mut action_stack: ResMut<ActionStack>,
    mut redo_action_reader: EventReader<RedoEditActionEvent>,
    tilemap_storage: Query<&TileStorage>,
    tile_query: Query<&TileMaterial>,
) {
    redo_action_reader.iter().for_each(|_| {
        if let Some(action) = action_stack.redo() {
            let storage = tilemap_storage.single();
            let _ = commit_action(&mut commands, storage, &tile_query, &action);
        }
    });
}

pub fn undo_edit_action(
    mut commands: Commands,
    mut action_stack: ResMut<ActionStack>,
    mut undo_action_reader: EventReader<UndoEditActionEvent>,
    mut tilemap_storage: Query<&TileStorage>,
    mut tile_query: Query<&TileMaterial>,
) {
    undo_action_reader.iter().for_each(|_| {
        if let Some(action) = action_stack.undo() {
            let storage = tilemap_storage.single_mut();
            let _ = commit_action(&mut commands, storage, &tile_query, &action);
        }
    });
}

pub fn add_edit_actions(
    mut commands: Commands,
    mut action_stack: ResMut<ActionStack>,
    mut add_action_reader: EventReader<AddEditActionEvent>,
    tilemap_storage: Query<&TileStorage>,
    tile_query: Query<&TileMaterial>,
) {
    add_action_reader
        .iter()
        .for_each(|AddEditActionEvent(action)| match action {
            EditAction::PlaceTile { .. } => {
                let storage = tilemap_storage.single();
                let undo = commit_action(&mut commands, storage, &tile_query, &action);
                action_stack.push(action.clone(), undo);
            }
            _ => {}
        });
}

pub fn tile_click(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse_btn: Res<Input<MouseButton>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut add_edit_action: EventWriter<AddEditActionEvent>,
    brush: Res<RogBrush>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if !mouse_btn.just_released(MouseButton::Left) {
        return;
    }

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let cursor_pos = world_position.extend(1.0);
        for (size, grid_size, map_type, storage, transform) in tilemap_q.iter() {
            let cursor_in_map_pos: Vec2 = {
                let cursor_pos = Vec4::from((cursor_pos, 1.0));
                let cursor_in_map_pos = transform.compute_matrix().inverse() * cursor_pos;
                cursor_in_map_pos.xyz()
            }
            .truncate();

            if let Some(tile_pos) =
                TilePos::from_world_pos(&cursor_in_map_pos, size, grid_size, map_type)
            {
                add_edit_action.send(AddEditActionEvent(EditAction::PlaceTile {
                    material: brush.material.clone(),
                    tile_pos,
                    size: brush.size,
                }))
            }
        }
    }
}

pub fn setup_blank_level(mut commands: Commands, images: Res<ImageAssets>) {
    let map_size = TilemapSize { x: 32, y: 32 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let tile_size = TilemapTileSize { x: 8.0, y: 8.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(32),
                        ..default()
                    },
                    TileMaterial::Floor,
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(images.tilemap_image.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

pub fn teardown() {}

// /**
//  * lage en et eget set med plasserbare tiles for tomrom, vegger, gulv, etc.
//  * trykk på en tile så toggler man en enkel vegg tile
//  * lag et system som endrer tile sin teksturer etter hvilke vegger som er naboer
//  * lag et brush UI og state så man kan toggle brush av og på
//  * (gjør dette via events slik at man kan ha push og pop events for reversering av state)
//  * lag plasserbare materialer som man kan velge
//  * lag en flood fill brush
//  * lag et gulv materiale
//  * lag flere plasserbare ting som spawn points, kister, nøkler
//  * gjør det mulig å redigere farge på ting man plasserer
//  *
//  *
//  * lag egne cursors som angir valgt brush
//  * lag hurtigtaster for brusher
// RESSURS: actions
// ADD_ACTION_EVENT(Action) Action { TARGET, PREVIOUS_VALUE, NEW_VALUE}
// POP_ACTION_EVENT() // tar siste action og reverserer den ved å sette TARGET til previous value
//  */
