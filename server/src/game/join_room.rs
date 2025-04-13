use super::Handler;
use log::info;
use shared::{ErrorResponse, JoinRoomCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

impl Handler for JoinRoomCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        player_uuid: &mut Option<uuid::Uuid>,
        _party_start_notify: Arc<Notify>,
    ) -> shared::ServerAPIResponse {
        let room_uuid = self.0;
        if let Some(player_uuid) = player_uuid {
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
                    room.players.push(*player_uuid);
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
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_join_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

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
        let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

        let res = handle_request(
            &join_room_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let res = handle_request(
            &register_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

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

        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.get(&test_room_uuid).unwrap().players.len(), 2);

        if let ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(room_uuid)) = res {
            let room = gs.rooms.get(&room_uuid).unwrap();
            assert_eq!(room.uuid(), room_uuid);
            assert_eq!(*room.header.name, room_name);
            assert!(room.players.contains(&register_uuid));

            let player = gs.players.get(&register_uuid).unwrap();
            assert_eq!(player.room, Some(room_uuid));
        } else {
            panic!("Response type not matching");
        }

        // Drop game state Mutex lock
        std::mem::drop(gs);

        let res = handle_request(
            &join_room_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );
    }
}
