use self::{
    components::{Paused, PhysicsSet},
    effects::flick_system,
    systems::{
        animate_sprite, example_setup, example_update, game_keys, pause_controls, setup_player,
        teardown,
    },
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
        app.add_systems(OnEnter(GameState::InGame), (example_setup, setup_player))
            .add_systems(
                Update,
                (
                    game_keys,
                    animate_sprite,
                    example_update,
                    flick_system,
                    pause_controls,
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
