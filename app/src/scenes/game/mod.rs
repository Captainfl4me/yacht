use bevy::prelude::*;
use lobby::TopLevelLobbyScreen;

use crate::network::{events::RoomUpdateEvent, NetworkCommandEvent};

use super::{despawn_screen, AppState};

mod lobby;
mod round;
mod scoreboard;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    NotStarted,
    Started,
    Scoreboard,
    #[default]
    Disabled,
}

#[derive(Component)]
enum MenuButtonAction {
    StartGame,
}

pub fn game_plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_systems(OnEnter(AppState::Game), game_setup)
        .add_systems(OnEnter(GameState::NotStarted), lobby::lobby_setup)
        .add_systems(
            Update,
            (lobby::join_room_update, lobby::player_list_update)
                .run_if(in_state(GameState::NotStarted)),
        )
        .add_systems(
            Update,
            (menu_action, game_state_listener).run_if(in_state(AppState::Game)),
        )
        .add_systems(
            OnExit(GameState::NotStarted),
            despawn_screen::<TopLevelLobbyScreen>,
        );
}

fn game_setup(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::NotStarted);
}

type MenuActionInteractionQueryType<'a, 'b, 'c> =
    Query<'c, 'b, (&'a Interaction, &'a MenuButtonAction), (Changed<Interaction>, With<Button>)>;

fn menu_action(
    interaction_query: MenuActionInteractionQueryType,
    mut network_command: EventWriter<NetworkCommandEvent>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::StartGame => {
                    network_command.write(NetworkCommandEvent(
                        shared::ServerAPICommand::StartGame(shared::StartGameCommand),
                    ));
                }
            }
        }
    }
}

fn game_state_listener(
    mut game_state: ResMut<NextState<GameState>>,
    mut room_update_event: EventReader<RoomUpdateEvent>,
) {
    if let Some(RoomUpdateEvent(room_update_reason)) = room_update_event.read().next() {
        if let shared::RoomUpdateReason::GameStateChange(new_game_state) = room_update_reason.0 {
            game_state.set(match new_game_state {
                shared::GameState::NotStarted => GameState::NotStarted,
                shared::GameState::Started => GameState::Started,
                shared::GameState::Scoreboard => GameState::Scoreboard,
            });
        }
    }
}
