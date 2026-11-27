use super::Handler;
use log::info;
use shared::{ErrorResponse, JoinRoomCommand, PlayerHeader, ScoreSheet, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for JoinRoomCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        let room_uuid = self.0;
        if let Some(player_uuid) = &data.uuid {
            let mut gs = game_state.lock().await;

            if !gs.players.contains_key(player_uuid) {
                return ServerAPIResponse::Error(ErrorResponse::NotFound);
            }
            let player_name = gs.players.get(player_uuid).unwrap().name.clone();

            // A player can only be in one room at a time: reject joining a
            // second room, otherwise the old room would keep listing the
            // player while player.room points to the new room.
            if gs.players.get(player_uuid).unwrap().room.is_some() {
                return ServerAPIResponse::Error(ErrorResponse::PlayerAlreadyInRoom);
            }

            let room_opt = gs.rooms.get_mut(&room_uuid);

            if let Some(room) = room_opt {
                if room.started {
                    return ServerAPIResponse::Error(ErrorResponse::GameAlreadyStarted);
                }
                if room.players.len() >= shared::MAX_PLAYERS_PER_ROOM {
                    return ServerAPIResponse::Error(ErrorResponse::RoomFull);
                }

                let mut already_in_room = false;
                for id in room.players.iter() {
                    if player_uuid == id {
                        already_in_room = true;
                    }
                }

                if !already_in_room {
                    data.listen_change_room = Some(room.change_room.subscribe());
                    data.listen_change_dices = Some(room.change_dices.subscribe());
                    data.listen_change_turn = Some(room.change_turn.subscribe());
                    data.listen_change_dices_mask = Some(room.change_dices_mask.subscribe());
                    data.listen_change_score = Some(room.change_score.subscribe());
                    room.players.push(*player_uuid);
                    room.scores.push(ScoreSheet::default());
                    if let Err(e) =
                        room.change_room
                            .send(shared::RoomUpdateReason::NewPlayer(PlayerHeader {
                                uuid: *player_uuid,
                                name: player_name,
                            }))
                    {
                        log::warn!("Failed to broadcast NewPlayer: {}", e);
                    }
                    gs.players
                        .entry(*player_uuid)
                        .and_modify(|player| player.room = Some(room_uuid));

                    info!("Player {} join room {}", player_uuid, room_uuid);
                    ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(room_uuid))
                } else {
                    ServerAPIResponse::Error(ErrorResponse::PlayerAlreadyInRoom)
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
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_join_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = uuid::Uuid::new_v4();
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(shared::PROTOCOL_VERSION, register_uuid, "Test".to_string()));
        let room_name = "MyName".to_string();
        let test_room_uuid = {
            let mut gs = game_state.lock().await;
            let test_room = Room::new(uuid::Uuid::new_v4(), room_name.clone());
            let uuid = test_room.uuid();
            gs.rooms.insert(test_room.uuid(), test_room);

            uuid
        };

        let join_room_cmd = ServerAPICommand::JoinRoom(shared::JoinRoomCommand(test_room_uuid));
        let mut socket_data = SocketLinkedData::default();
        let res = handle_request(&join_room_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse(None)));

        let res = handle_request(&join_room_cmd, &game_state, &mut socket_data).await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));
        assert!(socket_data.listen_change_room.is_some());

        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.get(&test_room_uuid).unwrap().players.len(), 2);

        if let ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(room_uuid)) = res {
            let room = gs.rooms.get(&room_uuid).unwrap();
            assert_eq!(room.uuid(), room_uuid);
            assert_eq!(*room.header.name, room_name);
            assert!(room.players.contains(&register_uuid));
            assert_eq!(room.scores.len(), 2);

            assert!(socket_data.listen_change_room.is_some());
            assert!(socket_data.listen_change_dices.is_some());
            assert!(socket_data.listen_change_dices_mask.is_some());
            assert!(socket_data.listen_change_turn.is_some());
            assert!(socket_data.listen_change_score.is_some());

            let player = gs.players.get(&register_uuid).unwrap();
            assert_eq!(player.room, Some(room_uuid));
        } else {
            panic!("Response type not matching");
        }

        // Drop game state Mutex lock
        std::mem::drop(gs);

        let res = handle_request(&join_room_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );
    }

    #[tokio::test]
    async fn test_join_room_rejects_started_game() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let creator_uuid = Uuid::new_v4();
        let joiner_uuid = Uuid::new_v4();
        let mut creator_data = SocketLinkedData::default();
        let mut joiner_data = SocketLinkedData::default();

        async fn register(
            game_state: &Arc<Mutex<GameState>>,
            uuid: Uuid,
            name: &str,
            data: &mut SocketLinkedData,
        ) {
            handle_request(
                &ServerAPICommand::Register(shared::RegisterCommand(
                    shared::PROTOCOL_VERSION,
                    uuid,
                    name.to_string(),
                )),
                game_state,
                data,
            )
            .await;
        }
        register(&game_state, creator_uuid, "Alice", &mut creator_data).await;
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
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            &game_state,
            &mut creator_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        register(&game_state, joiner_uuid, "Bob", &mut joiner_data).await;
        let res = handle_request(
            &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
            &game_state,
            &mut joiner_data,
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::GameAlreadyStarted)
        );
    }

    #[tokio::test]
    async fn test_join_room_rejects_full_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let creator_uuid = Uuid::new_v4();
        let mut creator_data = SocketLinkedData::default();

        handle_request(
            &ServerAPICommand::Register(shared::RegisterCommand(
                shared::PROTOCOL_VERSION,
                creator_uuid,
                "Alice".to_string(),
            )),
            &game_state,
            &mut creator_data,
        )
        .await;
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

        for i in 0..shared::MAX_PLAYERS_PER_ROOM - 1 {
            let uuid = Uuid::new_v4();
            let mut data = SocketLinkedData::default();
            handle_request(
                &ServerAPICommand::Register(shared::RegisterCommand(
                    shared::PROTOCOL_VERSION,
                    uuid,
                    format!("P{}", i),
                )),
                &game_state,
                &mut data,
            )
            .await;
            let res = handle_request(
                &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
                &game_state,
                &mut data,
            )
            .await;
            assert!(
                matches!(
                    res,
                    ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
                ),
                "join {} should succeed",
                i
            );
        }

        let last_uuid = Uuid::new_v4();
        let mut last_data = SocketLinkedData::default();
        handle_request(
            &ServerAPICommand::Register(shared::RegisterCommand(
                shared::PROTOCOL_VERSION,
                last_uuid,
                "Last".to_string(),
            )),
            &game_state,
            &mut last_data,
        )
        .await;
        let res = handle_request(
            &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
            &game_state,
            &mut last_data,
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::RoomFull)
        );
    }

    #[tokio::test]
    async fn test_join_unknown_room_returns_not_found() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let uuid = Uuid::new_v4();
        super::super::test_utils::register(&game_state, &mut socket_data, uuid, "Alice").await;

        let res =
            super::super::test_utils::join_room(&game_state, &mut socket_data, Uuid::new_v4()).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );
    }

    #[tokio::test]
    async fn test_join_second_room_rejected() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let (_creator_data, mut joiner_data, _c, joiner_uuid, room1) =
            super::super::test_utils::setup_room(&game_state, false).await;

        // A third player creates a second room
        let third_uuid = Uuid::new_v4();
        let mut third_data = SocketLinkedData::default();
        super::super::test_utils::register(&game_state, &mut third_data, third_uuid, "Carol").await;
        let room2 =
            super::super::test_utils::create_room(&game_state, &mut third_data, "Room2").await;

        // The joiner is already in room1 and must not be able to join room2
        let res = super::super::test_utils::join_room(&game_state, &mut joiner_data, room2).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );

        // State must be unchanged: joiner still only in room1, room2 only has its creator
        let gs = game_state.lock().await;
        assert_eq!(gs.players.get(&joiner_uuid).unwrap().room, Some(room1));
        assert_eq!(gs.rooms.get(&room2).unwrap().players, vec![third_uuid]);
        assert_eq!(gs.rooms.get(&room1).unwrap().players.len(), 2);
    }

    #[tokio::test]
    async fn test_join_unregistered_uuid_returns_not_found() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = super::super::test_utils::unregistered_socket_data(Uuid::new_v4());
        let res = super::super::test_utils::join_room(&game_state, &mut socket_data, Uuid::new_v4()).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );
    }

    #[tokio::test]
    async fn test_join_room_already_listed_despite_cleared_reference() {
        // Inconsistent state: the room lists the player but player.room was
        // cleared. The join must be rejected, not duplicate the player.
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let uuid = Uuid::new_v4();
        let mut data = SocketLinkedData::default();
        super::super::test_utils::register(&game_state, &mut data, uuid, "Alice").await;
        let room_uuid = super::super::test_utils::create_room(&game_state, &mut data, "R").await;
        {
            let mut gs = game_state.lock().await;
            gs.players.get_mut(&uuid).unwrap().room = None;
        }
        let res = super::super::test_utils::join_room(&game_state, &mut data, room_uuid).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );
        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.get(&room_uuid).unwrap().players, vec![uuid]);
    }

}
