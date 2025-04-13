use super::Handler;
use log::info;
use shared::{ErrorResponse, ServerAPIResponse, StartGameCommand};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

impl Handler for StartGameCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        player_uuid: &mut Option<uuid::Uuid>,
        _party_start_notify: Arc<Notify>,
    ) -> shared::ServerAPIResponse {
        if let Some(player_uuid) = player_uuid {
            info!("Start game");
            let mut gs = game_state.lock().await;

            if let Some(game_room_uuid) = gs.players.get(player_uuid).unwrap().room {
                let room = gs.rooms.get_mut(&game_room_uuid).unwrap();

                if room.creator == *player_uuid {
                    room.started = true;
                    for player in room.players.clone() {
                        gs.players
                            .get(&player)
                            .unwrap()
                            .party_start_notify
                            .as_ref()
                            .unwrap()
                            .notify_one();
                    }
                    ServerAPIResponse::Ok
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
                }
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotFound)
            }
        } else {
            ServerAPIResponse::Error(ErrorResponse::NotRegister)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState, Room};
    use super::*;
    use shared::ServerAPICommand;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_start_game() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

        let register_uuid = Uuid::new_v4();
        let creator_uuid = Uuid::new_v4();
        let room_name = "MyName".to_string();

        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand(room_name.clone()));
        let start_game_cmd = ServerAPICommand::StartGame(shared::StartGameCommand);
        let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

        // Test without registering
        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        // Create room for later testing (creator_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(creator_uuid));
        let res = handle_request(
            &register_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        let res = handle_request(
            &create_room_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        let test_room_uuid = {
            if let ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(room_uuid)) = res {
                room_uuid
            } else {
                panic!("Response type not matching");
            }
        };

        // Testing endpoint guard with new user (register_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let join_room_cmd = ServerAPICommand::JoinRoom(shared::JoinRoomCommand(test_room_uuid));
        let res = handle_request(
            &register_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );

        let res = handle_request(
            &join_room_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)
        );

        // Testing endpoint (creator_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(creator_uuid));
        let res = handle_request(
            &register_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        party_started_notify.notified().await;
    }
}
