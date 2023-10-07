use crate::{game::prelude::MainCamera, get_some, GameState, ImageAssets};
use bevy::{math::Vec4Swizzles, prelude::*, reflect::Tuple, transform::commands};
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::{self, Neighbors, SquareDirection},
    prelude::*,
};

pub struct EditorPlugin;

mod components;
use components::*;
mod ui;
use ui::*;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InEditor), setup_blank_level)
            .add_event::<EditEvent>()
            .add_event::<UndoEditEvent>()
            .add_event::<RedoEditEvent>()
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
                )
                    .chain()
                    .run_if(in_state(GameState::InEditor)),
            )
            .add_systems(
                Update,
                (brush_panel_ui, editor_indicator_ui, brush_mode_ui)
                    .run_if(in_state(GameState::InEditor)),
            )
            .add_systems(OnExit(GameState::InEditor), teardown)
            .insert_resource(RogBrush::default())
            .insert_resource(ActionStack::default());
    }
}

pub fn key_input(
    keyboard: Res<Input<KeyCode>>,
    mut undo_edit_action: EventWriter<UndoEditEvent>,
    mut redo_edit_action: EventWriter<RedoEditEvent>,
) {
    if keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keyboard.just_released(KeyCode::Z) {
                redo_edit_action.send(RedoEditEvent);
            }
        } else {
            if keyboard.just_released(KeyCode::Z) {
                undo_edit_action.send(UndoEditEvent);
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
                                    TileMaterial::PlayerSpawn => false,
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
            TileMaterial::PlayerSpawn => {
                tiles_to_update.push(TileMapIndex::PlayerSpawn);
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
    mut redo_action_reader: EventReader<RedoEditEvent>,
    tilemap_storage: Query<&TileStorage>,
    tile_query: Query<&TileMaterial>,
) {
    redo_action_reader.iter().for_each(|_| {
        if let Some(actions) = action_stack.redo() {
            let storage = tilemap_storage.single();
            actions.iter().for_each(|action| {
                let _ = commit_action(&mut commands, storage, &tile_query, &action);
            })
        }
    });
}

pub fn undo_edit_action(
    mut commands: Commands,
    mut action_stack: ResMut<ActionStack>,
    mut undo_action_reader: EventReader<UndoEditEvent>,
    mut tilemap_storage: Query<&TileStorage>,
    mut tile_query: Query<&TileMaterial>,
) {
    undo_action_reader.iter().for_each(|_| {
        if let Some(actions) = action_stack.undo() {
            let storage = tilemap_storage.single_mut();
            actions.iter().for_each(|action| {
                let _ = commit_action(&mut commands, storage, &tile_query, &action);
            });
        }
    });
}

pub fn add_edit_actions(
    mut commands: Commands,
    mut action_stack: ResMut<ActionStack>,
    mut add_action_reader: EventReader<EditEvent>,
    tilemap_storage: Query<&TileStorage>,
    tile_query: Query<&TileMaterial>,
) {
    add_action_reader.iter().for_each(|EditEvent(actions)| {
        let undos: Vec<EditAction> =
            actions
                .iter()
                .fold(Vec::new(), |mut undos, action| match action {
                    EditAction::PlaceTile { .. } => {
                        let storage = tilemap_storage.single();
                        let undo = commit_action(&mut commands, storage, &tile_query, &action);

                        undos.push(undo);

                        undos
                    }
                    _ => undos,
                });
        action_stack.push(actions.clone(), undos);
    });
}

pub fn tile_click(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse_btn: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut add_edit_action: EventWriter<EditEvent>,
    stack: Res<ActionStack>,
    brush: Res<RogBrush>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if !mouse_btn.just_released(MouseButton::Left) {
        return;
    }

    let world_position = get_some!(window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor)));

    let cursor_pos = world_position.extend(1.0);

    // Do this for each tilemap. Might have more in the future (like a minimap)
    for (size, grid_size, map_type, storage, transform) in tilemap_q.iter() {
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xyz()
        }
        .truncate();

        let tile_pos = get_some!(TilePos::from_world_pos(
            &cursor_in_map_pos,
            size,
            grid_size,
            map_type
        ));

        // Single tile placement
        if !keyboard.pressed(KeyCode::ShiftLeft) {
            add_edit_action.send(EditEvent(vec![EditAction::PlaceTile {
                material: brush.material.clone(),
                tile_pos,
                size: brush.size,
            }]));
            return;
        }

        // Range tile placement. todo simplify
        let ltile_pos = get_some!(stack.last_tilepos());

        let mut actions = Vec::new();
        let range_x = if ltile_pos.x <= tile_pos.x {
            ltile_pos.x..=tile_pos.x
        } else {
            tile_pos.x..=ltile_pos.x
        };
        let range_y = if ltile_pos.y <= tile_pos.y {
            ltile_pos.y..=tile_pos.y
        } else {
            tile_pos.y..=ltile_pos.y
        };
        if range_x.clone().count() > range_y.clone().count() {
            for x in range_x {
                let tile_pos = TilePos { x, y: tile_pos.y };
                actions.push(EditAction::PlaceTile {
                    material: brush.material.clone(),
                    tile_pos,
                    size: brush.size,
                });
            }
        } else {
            for y in range_y.clone() {
                let tile_pos = TilePos { x: tile_pos.x, y };
                actions.push(EditAction::PlaceTile {
                    material: brush.material.clone(),
                    tile_pos,
                    size: brush.size,
                });
            }
        }

        add_edit_action.send(EditEvent(actions));
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
                    Name::new("Base Tilemap Tile"),
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
