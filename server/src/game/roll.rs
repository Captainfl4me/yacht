use super::Handler;
use log::info;
use shared::{ErrorResponse, RollCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for RollCommand {
    async fn handle_request(
        &self,
        _game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if let Some(_) = &data.uuid {
            info!("Receive Roll");
            ServerAPIResponse::Roll(shared::RollResponse)
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
    use crate::SocketLinkedData;

    #[tokio::test]
    async fn test_roll() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = Uuid::new_v4();
        let creator_uuid = Uuid::new_v4();
        let room_name = "MyName".to_string();

        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand(room_name.clone()));
        let start_game_cmd = ServerAPICommand::StartGame(shared::StartGameCommand);
        let mut socket_data = SocketLinkedData {
            uuid: None,
            listen_change_game_state: None,
            listen_change_turn: None,
            listen_change_dices_mask: None,
            listen_change_dices: None,
        };

        // Test without registering
        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut socket_data,
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
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        let res = handle_request(
            &create_room_cmd,
            &game_state,
            &mut socket_data,
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
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );

        let res = handle_request(
            &join_room_cmd,
            &game_state,
            &mut socket_data,
        )
        .await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut socket_data,
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
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(
            &start_game_cmd,
            &game_state,
            &mut socket_data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);

        socket_data.listen_change_game_state.unwrap().recv().await;

        todo!();
    }
}
