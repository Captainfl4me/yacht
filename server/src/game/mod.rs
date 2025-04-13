use log::*;
use shared::{ErrorResponse, RoomHeader, ServerAPICommand, ServerAPIResponse};
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

pub struct Player {
    uuid: Uuid,
    room: Option<Uuid>,
    pub connected: bool,
    pub party_start_notify: Option<Arc<Notify>>,
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

pub trait Handler {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<GameState>>,
        player_uuid: &mut Option<Uuid>,
        party_start_notify: Arc<Notify>,
    ) -> ServerAPIResponse;
}

mod create_room;
mod list_room;
mod ping;
mod register;
mod join_room;
mod start_game;

pub async fn handle_request(
    cmd: &ServerAPICommand,
    game_state: &Arc<Mutex<GameState>>,
    player_uuid: &mut Option<Uuid>,
    party_start_notify: Arc<Notify>,
) -> ServerAPIResponse {
    let res = match cmd {
        ServerAPICommand::Ping(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
        }
        ServerAPICommand::Register(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
        }
        ServerAPICommand::CreateRoom(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
        }
        ServerAPICommand::ListRoom(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
        }
        ServerAPICommand::JoinRoom(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
        }
        ServerAPICommand::StartGame(cmd) => {
            cmd.handle_request(game_state, player_uuid, party_start_notify.clone())
                .await
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
