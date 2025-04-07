use log::*;
use shared::{ErrorResponse, RoomHeader, ServerAPICommand, ServerAPIResponse};
use speedy::Readable;
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct Player {
    uuid: Uuid,
    room: Option<Uuid>,
    pub connected: bool,
}

#[derive(Clone)]
pub struct Room {
    header: RoomHeader,
    creator: Uuid,
    started: bool,
    turn: u8,
    players: Vec<Uuid>,
}
impl Room {
    pub fn new(creator: Uuid, name: String) -> Self {
        Room {
            header: RoomHeader {
                uuid: Uuid::new_v4(),
                name,
            },
            creator,
            started: false,
            turn: 0,
            players: vec![creator],
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.header.uuid
    }
    pub fn name(&self) -> &String {
        &self.header.name
    }
    pub fn creator(&self) -> Uuid {
        self.creator
    }
}

pub struct GameState {
    pub players: HashMap<Uuid, Player>,
    pub rooms: HashMap<Uuid, Room>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

pub async fn handle_request(
    cmd: &ServerAPICommand,
    game_state: &Arc<Mutex<GameState>>,
    player_uuid: &mut Option<Uuid>,
) -> ServerAPIResponse {
    let res = match cmd {
        ServerAPICommand::Ping(_ping_cmd) => {
            info!("Receive Ping");
            ServerAPIResponse::Ping(shared::PingResponse)
        }
        ServerAPICommand::Register(shared::RegisterCommand(uuid)) => {
            info!("Receive UUID: {}", uuid);
            *player_uuid = Some(*uuid);
            let mut gs = game_state.lock().await;
            gs.players
                .entry(*uuid)
                .or_insert(Player {
                    uuid: *uuid,
                    room: None,
                    connected: false,
                })
                .connected = true;

            ServerAPIResponse::Register(shared::RegisterResponse)
        }
        ServerAPICommand::CreateRoom(shared::CreateRoomCommand(create_room_name)) => {
            if let Some(player_uuid) = player_uuid {
                info!("Receive CreateRoom");
                let mut gs = game_state.lock().await;

                if gs.players.contains_key(player_uuid) {
                    if gs.players.get(player_uuid).unwrap().room.is_none() {
                        let new_room = Room::new(*player_uuid, create_room_name.clone());

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
        ServerAPICommand::ListRoom(list_room_cmd) => {
            if let Some(player_uuid) = player_uuid {
                info!("Receive ListRoom");
                let gs = game_state.lock().await;
                let room_list = gs.rooms.values().map(|room| room.header.clone()).collect();
                ServerAPIResponse::ListRoom(shared::ListRoomResponse(room_list))
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotRegister)
            }
        }
        ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)) => {
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
                            .and_modify(|player| player.room = Some(*room_uuid));
                        ServerAPIResponse::JoinRoom(shared::JoinRoomResponse(*room_uuid))
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
        ServerAPICommand::Roll(roll_cmd) => {
            if let Some(player_uuid) = player_uuid {
                info!("Receive Roll");
                ServerAPIResponse::Roll(shared::RollResponse)
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotRegister)
            }
        }
        ServerAPICommand::KeepDice(keep_dice_cmd) => {
            if let Some(player_uuid) = player_uuid {
                info!("Receive KeepDice");
                ServerAPIResponse::KeepDice(shared::KeepDiceResponse)
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotRegister)
            }
        }
    };

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;
        let cmd = ServerAPICommand::Ping(shared::PingCommand);

        let res = handle_request(&cmd, &game_state, &mut uuid).await;

        assert_eq!(res, ServerAPIResponse::Ping(shared::PingResponse));
    }

    #[tokio::test]
    async fn test_register() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

        let register_uuid = uuid::Uuid::new_v4();
        let cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));

        let res = handle_request(&cmd, &game_state, &mut uuid).await;

        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        assert_eq!(uuid, Some(register_uuid));

        let res = handle_request(&cmd, &game_state, &mut uuid).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
    }

    #[tokio::test]
    async fn test_create_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

        let register_uuid = uuid::Uuid::new_v4();
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let room_name = "MyName".to_string();
        let create_room_cmd =
            ServerAPICommand::CreateRoom(shared::CreateRoomCommand(room_name.clone()));

        // Test create room guard
        let res = handle_request(&create_room_cmd, &game_state, &mut uuid).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let res = handle_request(&register_cmd, &game_state, &mut uuid).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&create_room_cmd, &game_state, &mut uuid).await;
        assert!(matches!(
            res,
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(_))
        ));

        let gs = game_state.lock().await;
        assert_eq!(gs.rooms.len(), 1);

        if let ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(room_uuid)) = res {
            let room = gs.rooms.get(&room_uuid).unwrap();
            assert_eq!(room.uuid(), room_uuid);
            assert_eq!(*room.name(), room_name);

            let player = gs.players.get(&register_uuid).unwrap();
            assert_eq!(player.room, Some(room_uuid));
        }

        // Drop game state Mutex lock
        std::mem::drop(gs);

        let res = handle_request(&create_room_cmd, &game_state, &mut uuid).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::PlayerAlreadyInRoom)
        );
    }

    #[tokio::test]
    async fn test_list_room() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

        let register_uuid = uuid::Uuid::new_v4();
        let register_cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let room_name = "MyName".to_string();
        let mut test_room_uuid: Uuid;
        let list_room_cmd = ServerAPICommand::ListRoom(shared::ListRoomCommand);

        {
            let mut gs = game_state.lock().await;
            let test_room = Room::new(Uuid::new_v4(), room_name.clone());
            test_room_uuid = test_room.uuid();
            gs.rooms.insert(test_room_uuid, test_room);
        }

        let res = handle_request(&list_room_cmd, &game_state, &mut uuid).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::NotRegister)
        );

        let res = handle_request(&register_cmd, &game_state, &mut uuid).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));

        let res = handle_request(&list_room_cmd, &game_state, &mut uuid).await;
        assert!(matches!(
            res,
            ServerAPIResponse::ListRoom(shared::ListRoomResponse(_))
        ));

        if let ServerAPIResponse::ListRoom(shared::ListRoomResponse(rooms)) = res {
            assert_eq!(rooms.len(), 1);
            assert_eq!(rooms[0].uuid, test_room_uuid);
            assert_eq!(rooms[0].name, room_name);
        }
    }
}
