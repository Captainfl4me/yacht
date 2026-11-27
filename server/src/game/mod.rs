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
    /// Bumped on every registration; a socket only owns the player while its
    /// epoch matches, so stale sockets (e.g. a closed tab) cannot disconnect
    /// the player for a newer socket.
    pub epoch: u64,
    /// When the owning socket disconnected. Room membership is kept until the
    /// disconnect grace period expires (see `sweep_disconnected_players`).
    pub disconnected_at: Option<std::time::Instant>,
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
    /// How long a disconnected player keeps its room seat before being
    /// removed. Overridable via `YATCH_DISCONNECT_GRACE_SECS` (used by e2e).
    pub disconnect_grace: std::time::Duration,
}

impl GameState {
    pub fn new() -> Self {
        let grace_secs = std::env::var("YATCH_DISCONNECT_GRACE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        GameState {
            players: HashMap::new(),
            rooms: HashMap::new(),
            disconnect_grace: std::time::Duration::from_secs(grace_secs),
        }
    }
}

/// Called when a socket closes. Only the socket holding the player's current
/// epoch marks the player disconnected; stale sockets (already replaced by a
/// re-register, e.g. a closed duplicate tab) are ignored. The player keeps
/// its room seat during the grace period so a page reload or a brief network
/// drop does not eject it from the room/game.
pub fn on_socket_disconnect(gs: &mut GameState, socket_data: &super::SocketLinkedData) {
    let Some(uuid) = socket_data.uuid else {
        return;
    };
    let Some(player) = gs.players.get_mut(&uuid) else {
        return;
    };
    if socket_data.epoch != Some(player.epoch) {
        log::debug!("Ignoring disconnect of stale socket for player {}", uuid);
        return;
    }
    player.connected = false;
    player.disconnected_at = Some(std::time::Instant::now());
}

/// Removes players whose disconnect grace period has expired from their room.
/// Called periodically by the server's sweeper task.
pub fn sweep_disconnected_players(gs: &mut GameState, now: std::time::Instant) {
    let stale: Vec<Uuid> = gs
        .players
        .iter()
        .filter(|(_, p)| {
            !p.connected
                && p.disconnected_at
                    .is_some_and(|t| now.duration_since(t) >= gs.disconnect_grace)
        })
        .map(|(u, _)| *u)
        .collect();
    for uuid in stale {
        if let Some(room_id) = gs.players.get(&uuid).and_then(|p| p.room) {
            log::info!("Disconnect grace expired for player {}, removing from room", uuid);
            remove_player_from_room(gs, &room_id, &uuid);
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
pub(crate) mod test_utils {
    use super::{handle_request, GameState};
    use crate::SocketLinkedData;
    use shared::{ServerAPICommand, ServerAPIResponse};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    pub fn register_cmd(uuid: Uuid, name: &str) -> ServerAPICommand {
        ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            uuid,
            name.to_string(),
        ))
    }

    pub async fn register(
        game_state: &Arc<Mutex<GameState>>,
        data: &mut SocketLinkedData,
        uuid: Uuid,
        name: &str,
    ) {
        let res = handle_request(&register_cmd(uuid, name), game_state, data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Register(shared::RegisterResponse(None))
        );
    }

    pub async fn create_room(
        game_state: &Arc<Mutex<GameState>>,
        data: &mut SocketLinkedData,
        name: &str,
    ) -> Uuid {
        let res = handle_request(
            &ServerAPICommand::CreateRoom(shared::CreateRoomCommand(name.to_string())),
            game_state,
            data,
        )
        .await;
        match res {
            ServerAPIResponse::CreateRoom(shared::CreateRoomResponse(uuid)) => uuid,
            other => panic!("expected CreateRoom, got {:?}", other),
        }
    }

    pub async fn join_room(
        game_state: &Arc<Mutex<GameState>>,
        data: &mut SocketLinkedData,
        room_uuid: Uuid,
    ) -> ServerAPIResponse {
        handle_request(
            &ServerAPICommand::JoinRoom(shared::JoinRoomCommand(room_uuid)),
            game_state,
            data,
        )
        .await
    }

    pub async fn start_game(game_state: &Arc<Mutex<GameState>>, data: &mut SocketLinkedData) {
        let res = handle_request(
            &ServerAPICommand::StartGame(shared::StartGameCommand),
            game_state,
            data,
        )
        .await;
        assert_eq!(res, ServerAPIResponse::Ok);
    }

    /// Registers a creator and a joiner, creates a room and optionally starts
    /// the game. Returns (creator_data, joiner_data, creator_uuid, joiner_uuid, room_uuid).
    pub async fn setup_room(
        game_state: &Arc<Mutex<GameState>>,
        start: bool,
    ) -> (SocketLinkedData, SocketLinkedData, Uuid, Uuid, Uuid) {
        let creator_uuid = Uuid::new_v4();
        let joiner_uuid = Uuid::new_v4();
        let mut creator_data = SocketLinkedData::default();
        let mut joiner_data = SocketLinkedData::default();
        register(game_state, &mut creator_data, creator_uuid, "Alice").await;
        let room_uuid = create_room(game_state, &mut creator_data, "Room").await;
        register(game_state, &mut joiner_data, joiner_uuid, "Bob").await;
        let res = join_room(game_state, &mut joiner_data, room_uuid).await;
        assert!(matches!(res, ServerAPIResponse::JoinRoom(_)));
        if start {
            start_game(game_state, &mut creator_data).await;
        }
        (creator_data, joiner_data, creator_uuid, joiner_uuid, room_uuid)
    }

    /// Socket data carrying a uuid that was never registered.
    pub fn unregistered_socket_data(uuid: Uuid) -> SocketLinkedData {
        SocketLinkedData {
            uuid: Some(uuid),
            ..Default::default()
        }
    }

    /// Registers a player and then points its `room` at a non-existent room
    /// (dangling reference). Returns the fake room id.
    pub async fn register_with_dangling_room(
        game_state: &Arc<Mutex<GameState>>,
        data: &mut SocketLinkedData,
        uuid: Uuid,
        name: &str,
    ) -> Uuid {
        register(game_state, data, uuid, name).await;
        let fake_room = Uuid::new_v4();
        let mut gs = game_state.lock().await;
        gs.players.get_mut(&uuid).unwrap().room = Some(fake_room);
        drop(gs);
        fake_room
    }

    /// Registers a creator, creates a room and starts the game, all on a
    /// single socket (solo room). Returns (socket_data, creator_uuid, room_uuid).
    pub async fn setup_started_solo(
        game_state: &Arc<Mutex<GameState>>,
    ) -> (SocketLinkedData, Uuid, Uuid) {
        let creator_uuid = Uuid::new_v4();
        let mut data = SocketLinkedData::default();
        register(game_state, &mut data, creator_uuid, "Alice").await;
        let room_uuid = create_room(game_state, &mut data, "Room").await;
        start_game(game_state, &mut data).await;
        (data, creator_uuid, room_uuid)
    }

    /// Drops all room broadcast receivers from a socket, simulating a room
    /// whose channels have no listeners (broadcast `send` then fails).
    pub fn drop_room_receivers(data: &mut SocketLinkedData) {
        data.listen_change_room = None;
        data.listen_change_turn = None;
        data.listen_change_score = None;
        data.listen_change_dices_mask = None;
        data.listen_change_dices = None;
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
            epoch: 0,
            disconnected_at: None,
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

    #[test]
    fn test_remove_player_not_in_room_returns_false() {
        let creator = Uuid::new_v4();
        let mut room = Room::new(creator, "Room".to_string());
        assert!(!room.remove_player(&Uuid::new_v4()));
        assert_eq!(room.players, vec![creator]);
    }

    #[test]
    fn test_migrate_creator_noop_when_creator_still_in_room() {
        let creator = Uuid::new_v4();
        let mut room = Room::new(creator, "Room".to_string());
        room.players.push(Uuid::new_v4());
        assert_eq!(room.migrate_creator(), None);
        assert_eq!(room.creator, creator);
    }

    #[test]
    fn test_migrate_creator_fails_when_room_empty() {
        let creator = Uuid::new_v4();
        let mut room = Room::new(creator, "Room".to_string());
        room.players.clear();
        assert_eq!(room.migrate_creator(), None);
        assert_eq!(room.creator, creator);
    }

    fn socket_data_with(uuid: Uuid, epoch: u64) -> crate::SocketLinkedData {
        crate::SocketLinkedData {
            uuid: Some(uuid),
            epoch: Some(epoch),
            ..Default::default()
        }
    }

    fn gs_with_room_player() -> (GameState, Uuid, Uuid) {
        let mut gs = GameState::new();
        let p = make_player("Solo");
        let uuid = p.uuid;
        gs.players.insert(uuid, p);
        let room = Room::new(uuid, "Room".to_string());
        let room_uuid = room.uuid();
        gs.players.get_mut(&uuid).unwrap().room = Some(room_uuid);
        gs.rooms.insert(room_uuid, room);
        (gs, uuid, room_uuid)
    }

    #[test]
    fn test_on_socket_disconnect_marks_disconnected_but_keeps_room() {
        let (mut gs, uuid, room_uuid) = gs_with_room_player();
        gs.players.get_mut(&uuid).unwrap().epoch = 3;

        on_socket_disconnect(&mut gs, &socket_data_with(uuid, 3));

        let player = gs.players.get(&uuid).unwrap();
        assert!(!player.connected);
        assert!(player.disconnected_at.is_some());
        // Room membership is kept during the grace period
        assert_eq!(player.room, Some(room_uuid));
        assert!(gs.rooms.contains_key(&room_uuid));
    }

    #[test]
    fn test_on_socket_disconnect_ignores_stale_epoch() {
        let (mut gs, uuid, room_uuid) = gs_with_room_player();
        gs.players.get_mut(&uuid).unwrap().epoch = 3;

        // A stale socket (old epoch, e.g. a closed duplicate tab) must not
        // disconnect the player for the newer socket.
        on_socket_disconnect(&mut gs, &socket_data_with(uuid, 2));

        let player = gs.players.get(&uuid).unwrap();
        assert!(player.connected);
        assert!(player.disconnected_at.is_none());
        assert_eq!(player.room, Some(room_uuid));
    }

    #[test]
    fn test_on_socket_disconnect_without_uuid_is_noop() {
        let (mut gs, uuid, _) = gs_with_room_player();
        on_socket_disconnect(&mut gs, &crate::SocketLinkedData::default());
        assert!(gs.players.get(&uuid).unwrap().connected);
    }

    #[test]
    fn test_sweep_removes_player_only_after_grace() {
        let (mut gs, uuid, room_uuid) = gs_with_room_player();
        gs.disconnect_grace = std::time::Duration::from_secs(60);
        gs.players.get_mut(&uuid).unwrap().connected = false;
        let now = std::time::Instant::now();
        gs.players.get_mut(&uuid).unwrap().disconnected_at = Some(now);

        // Within the grace period: still in the room
        sweep_disconnected_players(&mut gs, now + std::time::Duration::from_secs(30));
        assert!(gs.rooms.contains_key(&room_uuid));
        assert_eq!(gs.players.get(&uuid).unwrap().room, Some(room_uuid));

        // After the grace period: removed from the room (and room deleted,
        // since the player was the only member)
        sweep_disconnected_players(&mut gs, now + std::time::Duration::from_secs(61));
        assert!(!gs.rooms.contains_key(&room_uuid));
        assert_eq!(gs.players.get(&uuid).unwrap().room, None);
    }

    #[test]
    fn test_sweep_keeps_connected_players() {
        let (mut gs, uuid, room_uuid) = gs_with_room_player();
        gs.disconnect_grace = std::time::Duration::from_secs(1);
        sweep_disconnected_players(&mut gs, std::time::Instant::now() + std::time::Duration::from_secs(3600));
        assert!(gs.rooms.contains_key(&room_uuid));
        assert_eq!(gs.players.get(&uuid).unwrap().room, Some(room_uuid));
    }

    #[test]
    fn test_remove_player_from_missing_room_clears_dangling_reference() {
        let mut gs = GameState::new();
        let p = make_player("Ghost");
        let uuid = p.uuid;
        gs.players.insert(uuid, p);
        let fake_room = Uuid::new_v4();
        gs.players.get_mut(&uuid).unwrap().room = Some(fake_room);

        // Room does not exist: must not panic, just clear the dangling reference.
        remove_player_from_room(&mut gs, &fake_room, &uuid);

        assert_eq!(gs.players.get(&uuid).unwrap().room, None);
        assert!(gs.rooms.is_empty());
    }
}
