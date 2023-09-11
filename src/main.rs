use bevy::{
    asset::ChangeWatcher,
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    DefaultPlugins,
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiSettings;
use bevy_egui::{
    egui::{FontData, FontDefinitions, FontFamily},
    EguiContexts, EguiPlugin,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::RngPlugin;
use config::Debug;
use editor::EditorPlugin;
use game::{prelude::MainCamera, GamePlugin};
use main_menu::*;
use std::{env, process, time::Duration};

mod config;
mod editor;
mod game;
mod main_menu;

pub const SCREEN: Vec2 = Vec2::from_array([1024.0, 768.0]);
pub const DARK: Color = Color::rgb(0.191, 0.184, 0.156);
pub const LIGHT: Color = Color::rgb(0.852, 0.844, 0.816);

// Example: Easy loading of assets
#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 8.0, tile_size_y = 8.0, columns = 13, rows = 11))]
    #[asset(path = "textures/1BitRogueSet.png")]
    pub image_atlas: Handle<TextureAtlas>,
    #[asset(path = "textures/1BitRogueSet.png")]
    pub set_image: Handle<Image>,
}

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    MainMenu,
    InGame,
    InEditor,
}

/**
 * The configuration for the game loop. For cleanliness
 */
fn main() {
    // Possibility for program args
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = config::ProgramConfig::build(&args).unwrap_or_else(|err| {
        println!("A problem occured when parsing args: {err}");
        process::exit(1);
    });

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RUG-MAN".into(),
                    resolution: (SCREEN.x, SCREEN.y).into(),
                    present_mode: PresentMode::AutoNoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                watch_for_changes: Some(ChangeWatcher {
                    delay: Duration::from_millis(200),
                }),
                ..Default::default()
            }),
    )
    .add_state::<GameState>()
    .insert_resource(Debug(cfg.debug))
    .add_loading_state(
        LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::InEditor),
    )
    .add_collection_to_loading_state::<_, ImageAssets>(GameState::AssetLoading)
    .add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        RngPlugin::new().with_rng_seed(220718),
        EguiPlugin,
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        MainMenuPlugin,
        GamePlugin,
        EditorPlugin,
        TilemapPlugin,
    ))
    .add_systems(Startup, (spawn_camera, setup_fonts))
    .add_systems(Update, (zoom, move_camera));

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(DARK),
            },
            ..default()
        },
        MainCamera,
    ));
}

fn zoom(
    keyboard: Res<Input<KeyCode>>,
    mut q: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = q.single_mut();
    let mut log_scale = projection.scale.ln();

    if keyboard.pressed(KeyCode::Q) {
        log_scale += 0.05;
    }

    if keyboard.pressed(KeyCode::E) {
        log_scale -= 0.05;
    }

    // always ensure you end up with sane values
    // (pick an upper and lower bound for your application)
    projection.scale = log_scale.exp();
    projection.scale = projection.scale.clamp(0.1, 5.0);
}

fn move_camera(keyboard: Res<Input<KeyCode>>, mut q: Query<&mut Transform, With<MainCamera>>) {
    let mut transform = q.single_mut();

    if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
        transform.translation.x -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
        transform.translation.x += 1.0;
    }
    if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
        transform.translation.y += 1.0;
    }
    if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
        transform.translation.y -= 1.0;
    }
}

fn setup_fonts(mut contexts: EguiContexts) {
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "visitor".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/visitor.ttf")),
    ); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "visitor".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("visitor".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
}

pub fn window_resized(
    windows: Query<&Window>,
    mut q: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut egui_settings: ResMut<EguiSettings>,
) {
    let window = windows.single();
    let scale = SCREEN.x / window.width();
    for mut projection in q.iter_mut() {
        projection.scale = scale;
        egui_settings.scale_factor = (window.width() / SCREEN.x).into();
    }
}
