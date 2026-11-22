use super::Handler;
use log::info;
use shared::{ErrorResponse, RestartGameCommand, ScoreSheet, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for RestartGameCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if let Some(player_uuid) = &data.uuid {
            let mut gs = game_state.lock().await;
            if gs.players.contains_key(player_uuid) {
                if let Some(room_id) = gs.players.get(player_uuid).unwrap().room {
                    if gs.rooms.contains_key(&room_id) {
                        let room = gs.rooms.get_mut(&room_id).unwrap();
                        if room.creator == *player_uuid && room.started {
                            room.started = false;
                            room.turn = 0;
                            room.throw_cnt = 0;
                            room.dices = [0; 5];
                            room.dices_mask = [0; 5];
                            room.scores = room
                                .players
                                .iter()
                                .map(|_| ScoreSheet::default())
                                .collect();
                            if let Err(e) = room.change_room.send(
                                shared::RoomUpdateReason::GameStateChange(
                                    shared::GameState::NotStarted,
                                ),
                            ) {
                                log::warn!("Failed to broadcast GameStateChange: {}", e);
                            }
                            info!("Player {} restarted game in room {}", player_uuid, room_id);
                            ServerAPIResponse::Ok
                        } else {
                            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
                        }
                    } else {
                        ServerAPIResponse::Error(ErrorResponse::NotFound)
                    }
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotFound)
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
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;
    use uuid::Uuid;

    fn register_cmd(uuid: Uuid, name: &str) -> ServerAPICommand {
        ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            uuid,
            name.to_string(),
        ))
    }

    #[tokio::test]
    async fn test_restart_game_guards() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let restart_cmd = ServerAPICommand::RestartGame(shared::RestartGameCommand);
        let mut socket_data = SocketLinkedData::default();

        let res = handle_request(&restart_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let creator_uuid = Uuid::new_v4();
        let joiner_uuid = Uuid::new_v4();
        let mut creator_data = SocketLinkedData::default();
        let mut joiner_data = SocketLinkedData::default();

        handle_request(&register_cmd(creator_uuid, "Alice"), &game_state, &mut creator_data).await;
        let res = handle_request(
            &ServerAPICommand::CreateRoom(shared::CreateRoomCommand("Room".to_string())),
            &game_state,
            &mut creator_data,
        )
        .await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };

        // Game not started yet
        let res = handle_request(&restart_cmd, &game_state, &mut creator_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)
        );

        handle_request(&register_cmd(joiner_uuid, "Bob"), &game_state, &mut joiner_data).await;
        handle_request(
            &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
            &game_state,
            &mut joiner_data,
        )
        .await;

        // Not the creator
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            &game_state,
            &mut creator_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);
        let res = handle_request(&restart_cmd, &game_state, &mut joiner_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotEnoughPermission)
        );

        // Registered but not in a room
        let outsider_uuid = Uuid::new_v4();
        let mut outsider_data = SocketLinkedData::default();
        handle_request(&register_cmd(outsider_uuid, "Eve"), &game_state, &mut outsider_data).await;
        let res = handle_request(&restart_cmd, &game_state, &mut outsider_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );
    }

    #[tokio::test]
    async fn test_restart_game_resets_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let creator_uuid = Uuid::new_v4();

        handle_request(&register_cmd(creator_uuid, "Alice"), &game_state, &mut socket_data).await;
        let res = handle_request(
            &ServerAPICommand::CreateRoom(shared::CreateRoomCommand("Room".to_string())),
            &game_state,
            &mut socket_data,
        )
        .await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);
        assert_eq!(
            socket_data
                .listen_change_room
                .as_mut()
                .unwrap()
                .try_recv()
                .unwrap(),
            shared::RoomUpdateReason::GameStateChange(shared::GameState::Started)
        );

        // Roll and score something so the room is dirty
        let res = handle_request(
            &ServerAPICommand::Roll(shared::RollCommand),
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().dices = [1, 2, 3, 4, 5];
        }
        let res = handle_request(
            &ServerAPICommand::SelectPoints(shared::SelectPointsCommand(shared::Score::Aces(0))),
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        let res = handle_request(
            &ServerAPICommand::RestartGame(shared::RestartGameCommand),
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        let gs = game_state.lock().await;
        let room = gs.rooms.get(&room_uuid).unwrap();
        assert!(!room.started);
        assert_eq!(room.turn, 0);
        assert_eq!(room.throw_cnt, 0);
        assert_eq!(room.dices, [0; 5]);
        assert_eq!(room.scores.len(), 1);
        assert!(!room.scores[0].is_full());
        std::mem::drop(gs);

        assert_eq!(
            socket_data
                .listen_change_room
                .as_mut()
                .unwrap()
                .try_recv()
                .unwrap(),
            shared::RoomUpdateReason::GameStateChange(shared::GameState::NotStarted)
        );
    }
}
