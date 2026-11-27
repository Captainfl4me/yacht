use bevy::prelude::*;
use lobby::TopLevelLobbyScreen;
use round::TopLevelGameScreen;

use crate::network::events::{ChangeTurnEvent, LeaveRoomEvent, RoomInfoEvent, RoomUpdateEvent};
use shared::PlayerHeader;
use uuid::Uuid;

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

#[derive(Resource, Default)]
pub struct RoomPlayers(pub Vec<PlayerHeader>);

#[derive(Resource, Default)]
pub struct CurrentTurn(pub usize);

#[derive(Resource, Default)]
pub struct ThrowsUsed(pub u8);

#[derive(Resource, Default)]
pub struct FinalScores(pub Vec<shared::FinalScoreEntry>);

#[derive(Resource, Default)]
pub struct RoomCreator(pub Option<Uuid>);

/// Game phase received with the first RoomInfo after a re-register (page
/// reload / reconnect), consumed by `game_setup` so the client resumes in the
/// correct sub-scene instead of always landing in the lobby.
#[derive(Resource, Default)]
pub struct PendingRoomGameState(pub Option<shared::GameState>);

pub fn game_plugin(app: &mut App) {
    app.init_state::<GameState>()
        .init_resource::<RoomPlayers>()
        .init_resource::<CurrentTurn>()
        .init_resource::<ThrowsUsed>()
        .init_resource::<FinalScores>()
        .init_resource::<RoomCreator>()
        .init_resource::<PendingRoomGameState>()
        .init_resource::<round::MyScoreSheet>()
        .add_systems(OnEnter(AppState::Game), (game_setup, reset_game_resources))
        .add_systems(
            Update,
            resume_from_room_info.run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnEnter(GameState::NotStarted), lobby::lobby_setup)
        .add_systems(OnEnter(GameState::Started), round::round_setup)
        .add_systems(OnEnter(GameState::Scoreboard), scoreboard::scoreboard_setup)
        .add_systems(
            Update,
            (
                lobby::join_room_update,
                lobby::player_list_update,
                lobby::button_action,
                lobby::creator_update,
            )
                .run_if(in_state(GameState::NotStarted)),
        )
        .add_systems(
            Update,
            (
                scoreboard::scoreboard_update,
                scoreboard::scoreboard_button_action,
            )
                .run_if(in_state(GameState::Scoreboard)),
        )
        .add_systems(
            Update,
            (
                round::dices_throw_update,
                round::button_action,
                round::click_on_dice,
                round::click_on_score,
                round::turn_indicator_update,
                round::players_list_update,
                round::throw_button_gate,
                round::score_change_update,
                round::dice_spawn_anim,
                round::score_sheet_totals_update,
                round::throws_left_update,
            )
                .run_if(in_state(GameState::Started)),
        )
        .add_systems(
            Update,
            (room_players_listener, turn_listener, leave_room_listener)
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(Update, game_state_listener.run_if(in_state(AppState::Game)))
        .add_systems(
            OnExit(GameState::NotStarted),
            despawn_screen::<TopLevelLobbyScreen>,
        )
        .add_systems(
            OnExit(GameState::Started),
            despawn_screen::<TopLevelGameScreen>,
        )
        .add_systems(
            OnExit(GameState::Scoreboard),
            despawn_screen::<scoreboard::TopLevelScoreboardScreen>,
        );
}

fn game_setup(
    mut game_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<PendingRoomGameState>,
) {
    game_state.set(match pending.0.take() {
        Some(shared::GameState::Started) => GameState::Started,
        Some(shared::GameState::Scoreboard) => GameState::Scoreboard,
        _ => GameState::NotStarted,
    });
}

/// After a reconnect/re-register while still in a room, the server answers
/// RoomInfo with the current room state: enter the game scene directly instead
/// of showing the menu.
fn resume_from_room_info(
    mut app_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingRoomGameState>,
    mut room_info_event: EventReader<RoomInfoEvent>,
) {
    for RoomInfoEvent(info) in room_info_event.read() {
        info!("Resuming room {} (state {:?})", info.0.uuid, info.0.game_state);
        pending.0 = Some(info.0.game_state);
        app_state.set(AppState::Game);
    }
}

fn reset_game_resources(
    mut room_players: ResMut<RoomPlayers>,
    mut current_turn: ResMut<CurrentTurn>,
    mut throws_used: ResMut<ThrowsUsed>,
    mut my_score_sheet: ResMut<round::MyScoreSheet>,
    mut final_scores: ResMut<FinalScores>,
    mut room_creator: ResMut<RoomCreator>,
) {
    room_players.0.clear();
    current_turn.0 = 0;
    throws_used.0 = 0;
    my_score_sheet.0 = Default::default();
    final_scores.0.clear();
    room_creator.0 = None;
}

fn room_players_listener(
    mut room_players: ResMut<RoomPlayers>,
    mut room_creator: ResMut<RoomCreator>,
    mut final_scores: ResMut<FinalScores>,
    mut room_info_event: EventReader<RoomInfoEvent>,
    mut room_update_event: EventReader<RoomUpdateEvent>,
) {
    for RoomInfoEvent(info) in room_info_event.read() {
        room_creator.0 = Some(info.0.creator);
        room_players.0 = info.0.players.clone();
    }
    for RoomUpdateEvent(update) in room_update_event.read() {
        match &update.0 {
            shared::RoomUpdateReason::NewPlayer(player) => {
                if !room_players.0.iter().any(|p| p.uuid == player.uuid) {
                    room_players.0.push(player.clone());
                }
            }
            shared::RoomUpdateReason::PlayerLeft(uuid) => {
                room_players.0.retain(|p| p.uuid != *uuid);
            }
            shared::RoomUpdateReason::CreatorChanged(player) => {
                room_creator.0 = Some(player.uuid);
                if let Some(entry) = room_players.0.iter_mut().find(|p| p.uuid == player.uuid) {
                    *entry = player.clone();
                }
            }
            shared::RoomUpdateReason::FinalScores(entries) => {
                final_scores.0 = entries.clone();
            }
            _ => {}
        }
    }
}

fn leave_room_listener(
    mut app_state: ResMut<NextState<AppState>>,
    mut leave_room_event: EventReader<LeaveRoomEvent>,
) {
    for _ in leave_room_event.read() {
        app_state.set(AppState::Menu);
    }
}

fn turn_listener(
    mut current_turn: ResMut<CurrentTurn>,
    mut throws_used: ResMut<ThrowsUsed>,
    mut change_turn_event: EventReader<ChangeTurnEvent>,
) {
    for ChangeTurnEvent(turn) in change_turn_event.read() {
        current_turn.0 = turn.0 as usize;
        throws_used.0 = 0;
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
