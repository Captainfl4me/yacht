use super::Handler;
use log::info;
use rand::prelude::*;
use shared::{ErrorResponse, RollCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for RollCommand {
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
                        let turn = gs.rooms.get(&room_id).unwrap().turn;
                        let throw_cnt = gs.rooms.get(&room_id).unwrap().throw_cnt;
                        if gs.rooms.get(&room_id).unwrap().players[turn as usize] == *player_uuid
                            && throw_cnt < 3
                        {
                            info!("Receive Roll");
                            gs.rooms.entry(room_id).and_modify(|room| {
                                let mut rng = rand::rng();
                                let mut new_dices = room.dices;
                                for (id, mask) in room.dices_mask.iter().enumerate() {
                                    if *mask == 0 {
                                        let values: Vec<u8> = (1..=6).collect();
                                        new_dices[id] = *values.choose(&mut rng).unwrap();
                                    }
                                }
                                println!("{:?}", new_dices);
                                room.throw_cnt += 1;
                                room.dices = new_dices;
                                room.change_dices.send(new_dices).unwrap();
                            });
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
    use super::super::{handle_request, GameState, Room};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_roll() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = Uuid::new_v4();
        let creator_uuid = Uuid::new_v4();
        let room_name = "MyName".to_string();

        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand(room_name.clone()));
        let start_game_cmd = ServerAPICommand::StartGame(shared::StartGameCommand);
        let dice_select = [1, 0, 0, 1, 1];
        let keep_dice_cmd = ServerAPICommand::KeepDice(shared::KeepDiceCommand(dice_select));
        let roll_dice_cmd = ServerAPICommand::Roll(shared::RollCommand);
        let mut socket_data = SocketLinkedData::default();

        // Test without registering
        let res = handle_request(&roll_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        // Create room for later testing (creator_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(creator_uuid));
        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        let res = handle_request(&create_room_cmd, &game_state, &mut socket_data).await;
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
        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&roll_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotFound)
        );

        // Add new user to room (register_uuid)
        let res = handle_request(&join_room_cmd, &game_state, &mut socket_data).await;
        assert!(matches!(
            res,
            ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(_))
        ));

        // Start game (creator_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(creator_uuid));
        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&start_game_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        assert_eq!(
            socket_data
                .listen_change_game_state
                .as_mut()
                .unwrap()
                .try_recv()
                .unwrap(),
            shared::GameState::Started
        );

        // Test endpoint game (creator_uuid)
        let res = handle_request(&roll_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        let dices = socket_data
            .listen_change_dices
            .as_mut()
            .unwrap()
            .try_recv()
            .unwrap();

        // Test endpoint guard (register_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&roll_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission)
        );

        // Test endpoint game with some dice keep (creator_uuid)
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(creator_uuid));
        let res = handle_request(&register_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&keep_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        assert_eq!(
            socket_data
                .listen_change_dices_mask
                .as_mut()
                .unwrap()
                .try_recv()
                .unwrap(),
            dice_select
        );

        let res = handle_request(&roll_dice_cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Ok);

        let new_dices = socket_data
            .listen_change_dices
            .as_mut()
            .unwrap()
            .try_recv()
            .unwrap();

        for (id, mask) in dice_select.iter().enumerate() {
            if *mask == 1 {
                assert_eq!(dices[id], new_dices[id]);
            }
        }
    }
}
