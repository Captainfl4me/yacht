use crate::game::select_points::Score;

use super::Handler;
use log::info;
use shared::{ErrorResponse, JoinRoomCommand, ServerAPIResponse};
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
            info!("Receive JoinRoom");
            let mut gs = game_state.lock().await;

            let room_opt = gs.rooms.get_mut(&room_uuid);

            if let Some(room) = room_opt {
                let mut already_in_room = false;
                for id in room.players.iter() {
                    if player_uuid == id {
                        already_in_room = true;
                    }
                }

                if !already_in_room {
                    data.listen_change_game_state = Some(room.change_game_state.subscribe());
                    data.listen_change_dices = Some(room.change_dices.subscribe());
                    data.listen_change_turn = Some(room.change_turn.subscribe());
                    data.listen_change_dices_mask = Some(room.change_dices_mask.subscribe());
                    data.listen_change_score = Some(room.change_score.subscribe());
                    room.players.push(*player_uuid);
                    room.scores.push(Score::default());
                    gs.players
                        .entry(*player_uuid)
                        .and_modify(|player| player.room = Some(room_uuid));
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

    #[tokio::test]
    async fn test_join_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = uuid::Uuid::new_v4();
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
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
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&join_room_cmd, &game_state, &mut socket_data).await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));
        assert!(socket_data.listen_change_game_state.is_some());

        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.get(&test_room_uuid).unwrap().players.len(), 2);

        if let ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(room_uuid)) = res {
            let room = gs.rooms.get(&room_uuid).unwrap();
            assert_eq!(room.uuid(), room_uuid);
            assert_eq!(*room.header.name, room_name);
            assert!(room.players.contains(&register_uuid));
            assert_eq!(room.scores.len(), 2);

            assert!(socket_data.listen_change_game_state.is_some());
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
}
