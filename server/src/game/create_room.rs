use super::{Handler, Room};
use log::info;
use shared::{CreateRoomCommand, ErrorResponse, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for CreateRoomCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        let create_room_name = &self.0;
        if let Some(player_uuid) = &data.uuid {
            info!("Receive CreateRoom");
            let mut gs = game_state.lock().await;

            if gs.players.contains_key(player_uuid) {
                if gs.players.get(player_uuid).unwrap().room.is_none() {
                    let new_room = Room::new(*player_uuid, create_room_name.clone());

                    data.listen_change_game_state = Some(new_room.change_game_state.subscribe());
                    data.listen_change_dices = Some(new_room.change_dices.subscribe());
                    data.listen_change_turn = Some(new_room.change_turn.subscribe());
                    data.listen_change_dices_mask = Some(new_room.change_dices_mask.subscribe());
                    gs.rooms.insert(new_room.uuid(), new_room.clone());
                    gs.players
                        .entry(*player_uuid)
                        .and_modify(|p| p.room = Some(new_room.uuid()));

                    ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(new_room.uuid()))
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
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_create_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = uuid::Uuid::new_v4();
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let room_name = "MyName".to_string();
        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand(room_name.clone()));
        let mut socket_data = SocketLinkedData {
            uuid: None,
            listen_change_game_state: None,
            listen_change_turn: None,
            listen_change_dices_mask: None,
            listen_change_dices: None,
        };

        // Test create room guard
        let res = handle_request(&create_room_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&create_room_cmd, &game_state, &mut socket_data).await;
        assert!(socket_data.listen_change_game_state.is_some());
        assert!(matches!(
            res,
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(_))
        ));

        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.len(), 1);

        if let ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(room_uuid)) = res {
            let room = gs.rooms.get(&room_uuid).unwrap();
            assert_eq!(room.uuid(), room_uuid);
            assert_eq!(*room.header.name, room_name);
            assert!(room.players.contains(&register_uuid));
            assert_eq!(room.creator, register_uuid);

            let player = gs.players.get(&register_uuid).unwrap();
            assert_eq!(player.room, Some(room_uuid));
        } else {
            panic!("Response type not matching");
        }

        // Drop game state Mutex lock
        std::mem::drop(gs);

        let res = handle_request(&create_room_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );
    }
}
