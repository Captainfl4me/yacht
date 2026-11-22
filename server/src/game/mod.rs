use log::debug;
use shared::{PlayerHeader, RoomHeader, ScoreSheet, ServerAPICommand, ServerAPIResponse};
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

pub struct Player {
    #[allow(dead_code)]
    uuid: Uuid,
    pub name: String,
    pub room: Option<Uuid>,
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
    pub players: Vec<Uuid>,
    scores: Vec<ScoreSheet>,
    pub change_room: broadcast::Sender<shared::RoomUpdateReason>,
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
            scores: vec![ScoreSheet::default()],
            change_room: broadcast::channel(4).0,
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

    pub fn remove_player(&mut self, uuid: &Uuid) -> bool {
        if let Some(pos) = self.players.iter().position(|p| p == uuid) {
            self.players.remove(pos);
            if pos < self.scores.len() {
                self.scores.remove(pos);
            }
            if !self.players.is_empty() {
                if pos < self.turn as usize {
                    self.turn -= 1;
                }
                self.turn %= self.players.len() as u8;
            }
            true
        } else {
            false
        }
    }

    pub fn migrate_creator(&mut self) -> Option<Uuid> {
        if self.players.contains(&self.creator) {
            return None;
        }
        let new_creator = self.players.first().copied()?;
        self.creator = new_creator;
        Some(new_creator)
    }
}

pub fn remove_player_from_room(gs: &mut GameState, room_id: &Uuid, player_uuid: &Uuid) {
    let new_creator = match gs.rooms.get_mut(room_id) {
        Some(room) => {
            room.remove_player(player_uuid);
            if let Err(e) = room
                .change_room
                .send(shared::RoomUpdateReason::PlayerLeft(*player_uuid))
            {
                log::warn!("Failed to broadcast PlayerLeft: {}", e);
            }
            room.migrate_creator()
        }
        None => None,
    };

    if let Some(new_creator) = new_creator {
        if let Some(player) = gs.players.get(&new_creator) {
            let header = PlayerHeader {
                uuid: player.uuid,
                name: player.name.clone(),
            };
            if let Some(room) = gs.rooms.get_mut(room_id) {
                if let Err(e) = room
                    .change_room
                    .send(shared::RoomUpdateReason::CreatorChanged(header))
                {
                    log::warn!("Failed to broadcast CreatorChanged: {}", e);
                }
            }
        }
    }

    if let Some(player) = gs.players.get_mut(player_uuid) {
        player.room = None;
    }

    let room_empty = gs
        .rooms
        .get(room_id)
        .map(|room| room.players.is_empty())
        .unwrap_or(true);
    if room_empty {
        gs.rooms.remove(room_id);
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
mod leave_room;
mod list_room;
mod ping;
mod register;
mod restart_game;
mod roll;
mod room_info;
mod select_points;
mod start_game;

pub async fn handle_request(
    cmd: &ServerAPICommand,
    game_state: &Arc<Mutex<GameState>>,
    data: &mut super::SocketLinkedData,
) -> ServerAPIResponse {
    debug!("{:?}", cmd);
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
        ServerAPICommand::RoomInfo(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::LeaveRoom(cmd) => cmd.handle_request(game_state, data).await,
        ServerAPICommand::RestartGame(cmd) => cmd.handle_request(game_state, data).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player(name: &str) -> Player {
        Player {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            room: None,
            connected: true,
        }
    }

    #[test]
    fn test_remove_player_keeps_scores_and_turn_aligned() {
        let mut gs = GameState::new();
        let players: Vec<Uuid> = (0..3)
            .map(|i| {
                let p = make_player(&format!("P{}", i));
                let uuid = p.uuid;
                gs.players.insert(uuid, p);
                uuid
            })
            .collect();

        let mut room = Room::new(players[0], "Room".to_string());
        room.players = players.clone();
        room.scores = vec![ScoreSheet::default(), ScoreSheet::default(), ScoreSheet::default()];
        room.scores[2].aces = Some(5);
        room.turn = 2;
        room.started = true;
        let room_uuid = room.uuid();
        for uuid in &players {
            gs.players.get_mut(uuid).unwrap().room = Some(room_uuid);
        }
        gs.rooms.insert(room_uuid, room);

        // Remove first player while it is the third player's turn
        remove_player_from_room(&mut gs, &room_uuid, &players[0]);

        let room = gs.rooms.get(&room_uuid).unwrap();
        assert_eq!(room.players, vec![players[1], players[2]]);
        assert_eq!(room.scores.len(), 2);
        assert_eq!(room.scores[1].aces, Some(5));
        assert_eq!(room.turn, 1);
        assert_eq!(room.creator, players[1]);
        assert_eq!(gs.players.get(&players[0]).unwrap().room, None);
    }

    #[test]
    fn test_remove_last_player_deletes_room() {
        let mut gs = GameState::new();
        let p = make_player("Solo");
        let uuid = p.uuid;
        gs.players.insert(uuid, p);
        let room = Room::new(uuid, "Room".to_string());
        let room_uuid = room.uuid();
        gs.players.get_mut(&uuid).unwrap().room = Some(room_uuid);
        gs.rooms.insert(room_uuid, room);

        remove_player_from_room(&mut gs, &room_uuid, &uuid);
        assert!(gs.rooms.is_empty());
        assert_eq!(gs.players.get(&uuid).unwrap().room, None);
    }
}
