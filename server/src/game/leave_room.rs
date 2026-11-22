use super::Handler;
use log::info;
use shared::{ErrorResponse, LeaveRoomCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for LeaveRoomCommand {
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
                        super::remove_player_from_room(&mut gs, &room_id, player_uuid);

                        data.listen_change_room = None;
                        data.listen_change_dices = None;
                        data.listen_change_turn = None;
                        data.listen_change_dices_mask = None;
                        data.listen_change_score = None;

                        info!("Player {} left room {}", player_uuid, room_id);
                        ServerAPIResponse::LeaveRoom(shared::LeaveRoomResponse)
                    } else {
                        ServerAPIResponse::Error(ErrorResponse::NotFound)
                    }
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotInRoom)
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
    use shared::{PlayerHeader, ServerAPICommand};
    use uuid::Uuid;

    fn register_cmd(uuid: Uuid, name: &str) -> ServerAPICommand {
        ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            uuid,
            name.to_string(),
        ))
    }

    #[tokio::test]
    async fn test_leave_room_guards() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let leave_cmd = ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand);
        let mut socket_data = SocketLinkedData::default();

        let res = handle_request(&leave_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let uuid = Uuid::new_v4();
        let res = handle_request(&register_cmd(uuid, "Alice"), &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&leave_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotInRoom)
        );
    }

    #[tokio::test]
    async fn test_leave_room_removes_empty_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let creator_uuid = Uuid::new_v4();
        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand("MyRoom".to_string()));
        let leave_cmd = ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand);
        let mut socket_data = SocketLinkedData::default();

        handle_request(&register_cmd(creator_uuid, "Alice"), &game_state, &mut socket_data).await;
        let res = handle_request(&create_room_cmd, &game_state, &mut socket_data).await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };
        assert!(socket_data.has_all_handler());

        let res = handle_request(&leave_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::LeaveRoom(shared::LeaveRoomResponse)
        );
        assert!(!socket_data.has_all_handler());

        let gs = game_state.lock().await;
        assert!(gs.rooms.is_empty());
        assert_eq!(gs.players.get(&creator_uuid).unwrap().room, None);
        std::mem::drop(gs);

        let res = handle_request(&leave_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotInRoom)
        );
        let _ = room_uuid;
    }

    #[tokio::test]
    async fn test_leave_room_migrates_creator() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let creator_uuid = Uuid::new_v4();
        let joiner_uuid = Uuid::new_v4();
        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand("MyRoom".to_string()));
        let leave_cmd = ServerAPICommand::LeaveRoom(shared::LeaveRoomCommand);
        let mut creator_data = SocketLinkedData::default();
        let mut joiner_data = SocketLinkedData::default();

        handle_request(&register_cmd(creator_uuid, "Alice"), &game_state, &mut creator_data).await;
        let res = handle_request(&create_room_cmd, &game_state, &mut creator_data).await;
        let room_uuid = match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("Response type not matching: {:?}", other),
        };

        handle_request(&register_cmd(joiner_uuid, "Bob"), &game_state, &mut joiner_data).await;
        let join_cmd = ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid));
        let res = handle_request(&join_cmd, &game_state, &mut joiner_data).await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));
        // joiner receives NewPlayer broadcast
        assert!(joiner_data.listen_change_room.is_some());

        let res = handle_request(&leave_cmd, &game_state, &mut creator_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::LeaveRoom(shared::LeaveRoomResponse)
        );

        let gs = game_state.lock().await;
        let room = gs.rooms.get(&room_uuid).unwrap();
        assert_eq!(room.players, vec![joiner_uuid]);
        assert_eq!(room.creator, joiner_uuid);
        assert_eq!(room.scores.len(), 1);
        assert_eq!(gs.players.get(&creator_uuid).unwrap().room, None);
        std::mem::drop(gs);

        // joiner should have received PlayerLeft then CreatorChanged
        let mut reasons = Vec::new();
        while let Ok(reason) = joiner_data.listen_change_room.as_mut().unwrap().try_recv() {
            reasons.push(reason);
        }
        assert!(reasons.contains(&shared::RoomUpdateReason::PlayerLeft(creator_uuid)));
        assert!(reasons.contains(&shared::RoomUpdateReason::CreatorChanged(PlayerHeader {
            uuid: joiner_uuid,
            name: "Bob".to_string(),
        })));
    }
}
