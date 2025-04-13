use super::Handler;
use log::info;
use shared::{ErrorResponse, ListRoomCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

impl Handler for ListRoomCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        player_uuid: &mut Option<uuid::Uuid>,
        _party_start_notify: Arc<Notify>,
    ) -> shared::ServerAPIResponse {
        if player_uuid.is_some() {
            info!("Receive ListRoom");
            let gs = game_state.lock().await;
            let room_list = gs.rooms.values().map(|room| room.header.clone()).collect();
            ServerAPIResponse::ListRoom(shared::ListRoomResponse(room_list))
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
    async fn test_list_room() {
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
        let list_room_cmd = ServerAPICommand::ListRoom(shared::ListRoomCommand);
        let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

        let res = handle_request(
            &list_room_cmd,
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
            &list_room_cmd,
            &game_state,
            &mut uuid,
            party_started_notify.clone(),
        )
        .await;
        assert!(matches!(
            res,
            ServerAPIResponse::ListRoom(shared::ListRoomResponse(_))
        ));

        if let ServerAPIResponse::ListRoom(shared::ListRoomResponse(rooms)) = res {
            assert_eq!(rooms.len(), 1);
            assert_eq!(rooms[0].uuid, test_room_uuid);
            assert_eq!(rooms[0].name, room_name);
        } else {
            panic!("Response type not matching");
        }
    }
}
