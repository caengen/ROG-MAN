use crate::{game::prelude::MainCamera, GameState, ImageAssets};
use bevy::{math::Vec4Swizzles, prelude::*, reflect::Tuple};
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText},
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
                    editor_indicator_ui,
                    toggle_game_mode,
                    tile_click,
                    add_edit_actions,
                )
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

pub fn key_input(
    keyboard: Res<Input<KeyCode>>,
    mut undo_edit_action: EventWriter<UndoEditActionEvent>,
    mut redo_edit_action: EventWriter<RedoEditActionEvent>,
) {
    if keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]) {
        if (keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])) {
            if keyboard.just_pressed(KeyCode::Z) {
                redo_edit_action.send(RedoEditActionEvent);
            }
        }
        if keyboard.just_pressed(KeyCode::Z) {
            undo_edit_action.send(UndoEditActionEvent);
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

pub fn add_edit_actions(
    mut action_stack: ResMut<ActionStack>,
    mut add_action_reader: EventReader<AddEditActionEvent>,
    mut tilemap_storage: Query<&TileStorage>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    add_action_reader
        .iter()
        .for_each(|AddEditActionEvent(action)| match action {
            EditAction::PlaceTile {
                material,
                tile_pos,
                size,
            } => {
                let storage = tilemap_storage.single_mut();
                if let Some(entity) = storage.get(&tile_pos) {
                    match tile_query.get_mut(entity) {
                        Ok(mut index) => {
                            action_stack.push(
                                action.clone(),
                                EditAction::PlaceTile {
                                    tile_pos: *tile_pos,
                                    material: index_to_material(index.0),
                                    size: 1,
                                },
                            );

                            // Mutation
                            index.0 = material_to_index(&material);
                        }
                        Err(err) => {
                            println!("Entity does not exist: {}", err);
                        }
                    };
                }
            }
            _ => {}
        });
}

pub fn tile_click(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse_btn: Res<Input<MouseButton>>,
    mut tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut add_edit_action: EventWriter<AddEditActionEvent>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if !mouse_btn.just_pressed(MouseButton::Left) {
        return;
    }
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        let cursor_pos = world_position.extend(1.0);
        for (size, grid_size, map_type, storage, transform) in tilemap_q.iter_mut() {
            let cursor_in_map_pos: Vec2 = {
                let cursor_pos = Vec4::from((cursor_pos, 1.0));
                let cursor_in_map_pos = transform.compute_matrix().inverse() * cursor_pos;
                cursor_in_map_pos.xyz()
            }
            .truncate();

            if let Some(tile_pos) =
                TilePos::from_world_pos(&cursor_in_map_pos, size, grid_size, map_type)
            {
                add_edit_action.send(AddEditActionEvent(
                    (EditAction::PlaceTile {
                        material: TileMaterial::Wall,
                        tile_pos,
                        size: 1,
                    }),
                ))
            }
        }
    }
}

pub fn setup_blank_level(mut commands: Commands, images: Res<ImageAssets>) {
    let map_size = TilemapSize { x: 24, y: 24 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    let tile_size = TilemapTileSize { x: 8.0, y: 8.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(32),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(images.set_image.clone()),
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
