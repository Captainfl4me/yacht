use std::sync::Arc;

use shared::{
    ErrorResponse, PlayerHeader, RoomInfo, RoomInfoCommand, RoomInfoResponse, ServerAPIResponse,
};
use tokio::sync::Mutex;

use super::Handler;

impl Handler for RoomInfoCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if let Some(player_uuid) = &data.uuid {
            let gs = game_state.lock().await;
            if gs.players.contains_key(player_uuid) {
                if let Some(room_id) = gs.players.get(player_uuid).unwrap().room {
                    if gs.rooms.contains_key(&room_id) {
                        let players_uuid = gs.rooms.get(&room_id).unwrap().players.clone();

                        let room = gs.rooms.get(&room_id).unwrap();
                        ServerAPIResponse::RoomInfo(RoomInfoResponse(RoomInfo {
                            uuid: room_id,
                            name: room.header.name.clone(),
                            players: players_uuid
                                .iter()
                                .map(|uuid| PlayerHeader {
                                    uuid: *uuid,
                                    name: gs
                                        .players
                                        .get(uuid)
                                        .map(|p| p.name.clone())
                                        .unwrap_or_else(|| "Unknown".to_string()),
                                })
                                .collect(),
                            creator: room.creator,
                            game_state: if room.started {
                                shared::GameState::Started
                            } else {
                                shared::GameState::NotStarted
                            },
                        }))
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
    use super::super::test_utils::{create_room, register, setup_room};
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::{PlayerHeader, ServerAPICommand, ServerAPIResponse};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_room_info_guards() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut socket_data = SocketLinkedData::default();
        let cmd = ServerAPICommand::RoomInfo(RoomInfoCommand);

        // Not registered
        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Error(ErrorResponse::NotRegister));

        // Registered but not in a room
        let uuid = Uuid::new_v4();
        register(&game_state, &mut socket_data, uuid, "Alice").await;
        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Error(ErrorResponse::NotFound));
    }

    #[tokio::test]
    async fn test_room_info_returns_players_and_creator() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let (mut creator_data, mut joiner_data, creator_uuid, joiner_uuid, room_uuid) =
            setup_room(&game_state, false).await;

        let cmd = ServerAPICommand::RoomInfo(RoomInfoCommand);
        for data in [&mut creator_data, &mut joiner_data] {
            let res = handle_request(&cmd, &game_state, data).await;
            match res {
                ServerAPIResponse::RoomInfo(RoomInfoResponse(info)) => {
                    assert_eq!(info.uuid, room_uuid);
                    assert_eq!(info.name, "Room");
                    assert_eq!(info.creator, creator_uuid);
                    assert_eq!(
                        info.players,
                        vec![
                            PlayerHeader {
                                uuid: creator_uuid,
                                name: "Alice".to_string()
                            },
                            PlayerHeader {
                                uuid: joiner_uuid,
                                name: "Bob".to_string()
                            },
                        ]
                    );
                }
                other => panic!("expected RoomInfo, got {:?}", other),
            }
        }
    }

    #[tokio::test]
    async fn test_room_info_unknown_player_name_fallback() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let creator_uuid = Uuid::new_v4();
        let mut socket_data = SocketLinkedData::default();
        register(&game_state, &mut socket_data, creator_uuid, "Alice").await;
        let room_uuid = create_room(&game_state, &mut socket_data, "Room").await;

        // Simulate a player listed in the room but missing from the players map
        let ghost = Uuid::new_v4();
        {
            let mut gs = game_state.lock().await;
            gs.rooms.get_mut(&room_uuid).unwrap().players.push(ghost);
        }

        let res = handle_request(
            &ServerAPICommand::RoomInfo(RoomInfoCommand),
            &game_state,
            &mut socket_data,
        )
        .await;
        match res {
            ServerAPIResponse::RoomInfo(RoomInfoResponse(info)) => {
                let ghost_entry = info.players.iter().find(|p| p.uuid == ghost).unwrap();
                assert_eq!(ghost_entry.name, "Unknown");
            }
            other => panic!("expected RoomInfo, got {:?}", other),
        }
    }
}
