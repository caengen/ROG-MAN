use self::{
    components::{Paused, PhysicsSet},
    effects::flick_system,
    systems::{animate_sprite, game_indicator_ui, game_keys, teardown, toggle_edit_mode},
};
use crate::GameState;
use bevy::prelude::*;

mod collision;
mod components;
mod effects;
pub mod prelude;
mod systems;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::InGame))
        app.add_systems(
            Update,
            (
                game_keys,
                animate_sprite,
                flick_system,
                game_indicator_ui,
                toggle_edit_mode,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .configure_set(
            Update,
            PhysicsSet::Movement.before(PhysicsSet::CollisionDetection),
        )
        .add_systems(OnExit(GameState::InGame), teardown)
        .insert_resource(Paused(false));
    }
}
