use select_points::Score;
use shared::{RoomHeader, ServerAPICommand, ServerAPIResponse};
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

pub struct Player {
    #[allow(dead_code)]
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
    throw_cnt: u8,
    dices_mask: [u8; 5],
    dices: [u8; 5],
    players: Vec<Uuid>,
    scores: Vec<Score>,
    change_game_state: broadcast::Sender<shared::GameState>,
    change_turn: broadcast::Sender<usize>,
    change_score: broadcast::Sender<(u8, shared::Score)>,
    change_dices_mask: broadcast::Sender<[u8; 5]>,
    change_dices: broadcast::Sender<[u8; 5]>,
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
            throw_cnt: 0,
            players: vec![creator],
            scores: vec![Score::default()],
            change_game_state: broadcast::channel(4).0,
            change_turn: broadcast::channel(4).0,
            change_score: broadcast::channel(4).0,
            change_dices_mask: broadcast::channel(4).0,
            change_dices: broadcast::channel(4).0,
            dices_mask: [0; 5],
            dices: [0; 5],
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
        data: &mut super::SocketLinkedData,
    ) -> ServerAPIResponse;
}

mod create_room;
mod join_room;
mod keep_dice;
mod list_room;
mod ping;
mod register;
mod roll;
mod select_points;
mod start_game;

pub async fn handle_request(
    cmd: &ServerAPICommand,
    game_state: &Arc<Mutex<GameState>>,
    data: &mut super::SocketLinkedData,
) -> ServerAPIResponse {
    match cmd {
        ServerAPICommand::Ping(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::Register(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::CreateRoom(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::ListRoom(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::JoinRoom(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::StartGame(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::Roll(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::KeepDice(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::SelectPoints(cmd) => cmd.handle_request(game_state, data).await,
    }
}
