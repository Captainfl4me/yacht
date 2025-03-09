use log::*;
use shared::{ErrorResponse, ServerAPICommand, ServerAPIResponse};
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

pub struct GameState {
    pub players: HashMap<Uuid, Player>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
        }
    }
}

pub async fn handle_request(
    msg: &[u8],
    game_state: &Arc<Mutex<GameState>>,
    player_uuid: &mut Option<Uuid>,
) -> Option<ServerAPIResponse> {
    if let Ok(cmd) = ServerAPICommand::read_from_buffer(msg) {
        info!("RCV: {:?}", cmd);
        let res = match cmd {
            ServerAPICommand::Ping(ping_cmd) => {
                info!("Receive Ping");
                ServerAPIResponse::Ping(shared::PingResponse)
            }
            ServerAPICommand::Register(shared::RegisterCommand(uuid)) => {
                info!("Receive UUID: {}", uuid);
                *player_uuid = Some(uuid);
                let mut gs = game_state.lock().await;
                gs.players.entry(uuid).or_insert(Player { uuid, room: None, connected: false }).connected = true;

                ServerAPIResponse::Register(shared::RegisterResponse)
            }
            ServerAPICommand::CreateRoom(create_room_cmd) => {
                if let Some(player_uuid) = player_uuid {
                    info!("Receive CreateRoom");
                    ServerAPIResponse::CreateRoom(shared::CreateRoomResponse)
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotRegister)
                }
            }
            ServerAPICommand::ListRoom(list_room_cmd) => {
                if let Some(player_uuid) = player_uuid {
                    info!("Receive ListRoom");
                    ServerAPIResponse::ListRoom(shared::ListRoomResponse(Vec::new()))
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotRegister)
                }
            }
            ServerAPICommand::JoinRoom(join_room_cmd) => {
                if let Some(player_uuid) = player_uuid {
                    info!("Receive JoinRoom");
                    ServerAPIResponse::JoinRoom(shared::JoinRoomResponse)
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

        Some(res)
    } else {
        error!("Response cannot be parsed!");
        None
    }
}
