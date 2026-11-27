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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::events::{ChangeTurnEvent, LeaveRoomEvent, RoomInfoEvent, RoomUpdateEvent};
    use shared::{ChangeTurnResponse, LeaveRoomResponse, RoomInfo, RoomInfoResponse, RoomUpdateReason, RoomUpdateResponse};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_resource::<RoomPlayers>()
            .init_resource::<CurrentTurn>()
            .init_resource::<ThrowsUsed>()
            .init_resource::<FinalScores>()
            .init_resource::<RoomCreator>()
            .init_resource::<PendingRoomGameState>()
            .init_resource::<round::MyScoreSheet>()
            .add_event::<RoomInfoEvent>()
            .add_event::<RoomUpdateEvent>()
            .add_event::<ChangeTurnEvent>()
            .add_event::<LeaveRoomEvent>();
        app
    }

    fn settle(app: &mut App) {
        for _ in 0..4 {
            app.update();
        }
    }

    fn header(uuid: Uuid, name: &str) -> shared::PlayerHeader {
        shared::PlayerHeader {
            uuid,
            name: name.to_string(),
        }
    }

    #[test]
    fn test_resume_from_room_info_enters_game_with_pending_phase() {
        let mut app = test_app();
        app.add_systems(
            Update,
            resume_from_room_info.run_if(in_state(AppState::Menu)),
        );
        app.add_systems(OnEnter(AppState::Game), game_setup);

        // Put the app in the menu (as after a page reload).
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Menu);
        settle(&mut app);
        assert_eq!(app.world().resource::<State<AppState>>().get(), &AppState::Menu);

        // Server says we are still in a started room.
        let room_uuid = Uuid::new_v4();
        let me = header(Uuid::new_v4(), "me");
        app.world_mut().send_event(RoomInfoEvent(RoomInfoResponse(RoomInfo {
            uuid: room_uuid,
            name: "r".to_string(),
            players: vec![me.clone()],
            creator: me.uuid,
            game_state: shared::GameState::Started,
        })));
        settle(&mut app);

        assert_eq!(app.world().resource::<State<AppState>>().get(), &AppState::Game);
        // game_setup consumed the pending phase and entered the round scene.
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Started
        );
        assert!(app.world().resource::<PendingRoomGameState>().0.is_none());
    }

    #[test]
    fn test_game_setup_defaults_to_lobby_without_pending_phase() {
        let mut app = test_app();
        app.add_systems(OnEnter(AppState::Game), game_setup);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        settle(&mut app);
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::NotStarted
        );
    }

    #[test]
    fn test_room_players_listener_handles_info_and_updates() {
        let mut app = test_app();
        app.add_systems(Update, room_players_listener);

        let a = header(Uuid::new_v4(), "A");
        let b = header(Uuid::new_v4(), "B");
        app.world_mut().send_event(RoomInfoEvent(RoomInfoResponse(RoomInfo {
            uuid: Uuid::new_v4(),
            name: "r".to_string(),
            players: vec![a.clone()],
            creator: a.uuid,
            game_state: shared::GameState::NotStarted,
        })));
        app.update();
        assert_eq!(app.world().resource::<RoomPlayers>().0, vec![a.clone()]);
        assert_eq!(app.world().resource::<RoomCreator>().0, Some(a.uuid));

        // NewPlayer appends, duplicates are ignored.
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::NewPlayer(b.clone()),
        )));
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::NewPlayer(b.clone()),
        )));
        app.update();
        assert_eq!(
            app.world().resource::<RoomPlayers>().0,
            vec![a.clone(), b.clone()]
        );

        // PlayerLeft removes.
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::PlayerLeft(a.uuid),
        )));
        app.update();
        assert_eq!(app.world().resource::<RoomPlayers>().0, vec![b.clone()]);

        // CreatorChanged updates the creator.
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::CreatorChanged(b.clone()),
        )));
        app.update();
        assert_eq!(app.world().resource::<RoomCreator>().0, Some(b.uuid));

        // FinalScores are stored.
        let entries = vec![shared::FinalScoreEntry {
            player: b.clone(),
            total: 42,
        }];
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::FinalScores(entries.clone()),
        )));
        app.update();
        assert_eq!(app.world().resource::<FinalScores>().0, entries);
    }

    #[test]
    fn test_turn_listener_updates_turn_and_resets_throws() {
        let mut app = test_app();
        app.add_systems(Update, turn_listener);
        app.world_mut().resource_mut::<ThrowsUsed>().0 = 2;
        app.world_mut().send_event(ChangeTurnEvent(ChangeTurnResponse(1)));
        app.update();
        assert_eq!(app.world().resource::<CurrentTurn>().0, 1);
        assert_eq!(app.world().resource::<ThrowsUsed>().0, 0);
    }

    #[test]
    fn test_leave_room_listener_goes_back_to_menu() {
        let mut app = test_app();
        app.add_systems(Update, leave_room_listener);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        settle(&mut app);
        app.world_mut()
            .send_event(LeaveRoomEvent(LeaveRoomResponse));
        settle(&mut app);
        assert_eq!(app.world().resource::<State<AppState>>().get(), &AppState::Menu);
    }

    #[test]
    fn test_game_state_listener_maps_room_updates() {
        let mut app = test_app();
        app.add_systems(Update, game_state_listener);
        app.world_mut().send_event(RoomUpdateEvent(RoomUpdateResponse(
            RoomUpdateReason::GameStateChange(shared::GameState::Scoreboard),
        )));
        settle(&mut app);
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Scoreboard
        );
    }

    #[test]
    fn test_reset_game_resources_clears_state() {
        let mut app = test_app();
        app.add_systems(OnEnter(AppState::Game), reset_game_resources);
        app.world_mut().resource_mut::<RoomPlayers>().0 = vec![header(Uuid::new_v4(), "A")];
        app.world_mut().resource_mut::<CurrentTurn>().0 = 3;
        app.world_mut().resource_mut::<ThrowsUsed>().0 = 2;
        app.world_mut().resource_mut::<FinalScores>().0 = vec![shared::FinalScoreEntry {
            player: header(Uuid::new_v4(), "A"),
            total: 1,
        }];
        app.world_mut().resource_mut::<RoomCreator>().0 = Some(Uuid::new_v4());
        app.world_mut().resource_mut::<round::MyScoreSheet>().0.aces = Some(5);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        settle(&mut app);
        assert!(app.world().resource::<RoomPlayers>().0.is_empty());
        assert_eq!(app.world().resource::<CurrentTurn>().0, 0);
        assert_eq!(app.world().resource::<ThrowsUsed>().0, 0);
        assert!(app.world().resource::<FinalScores>().0.is_empty());
        assert!(app.world().resource::<RoomCreator>().0.is_none());
        assert!(app.world().resource::<round::MyScoreSheet>().0.aces.is_none());
    }
}
