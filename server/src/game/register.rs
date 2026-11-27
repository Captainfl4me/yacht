use super::{Handler, Player};
use log::info;
use shared::{ErrorResponse, RegisterCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

impl Handler for RegisterCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if self.0 != shared::PROTOCOL_VERSION {
            info!(
                "Protocol version mismatch: client {} server {}",
                self.0,
                shared::PROTOCOL_VERSION
            );
            return ServerAPIResponse::Error(ErrorResponse::VersionMismatch);
        }
        let uuid: Uuid = self.1;
        let name = self.2.clone();
        info!("Receive UUID: {} name: {}", uuid, name);
        let mut gs = game_state.lock().await;
        let (epoch, room_id) = {
            let player = gs.players.entry(uuid).or_insert(Player {
                uuid,
                name: name.clone(),
                room: None,
                connected: false,
                epoch: 0,
                disconnected_at: None,
            });
            // Each registration invalidates the previous socket for this
            // player: only the newest epoch may disconnect it.
            player.epoch += 1;
            player.connected = true;
            player.disconnected_at = None;
            player.name = name;
            (player.epoch, player.room)
        };
        data.uuid = Some(uuid);
        data.epoch = Some(epoch);

        // If the player is still in a room (e.g. after a page reload or a
        // brief network drop within the grace period), re-attach this socket
        // to the room's broadcast channels and tell the client which room it
        // is in so it can re-sync.
        if let Some(room_id) = room_id {
            if let Some(room) = gs.rooms.get(&room_id) {
                data.listen_change_room = Some(room.change_room.subscribe());
                data.listen_change_turn = Some(room.change_turn.subscribe());
                data.listen_change_score = Some(room.change_score.subscribe());
                data.listen_change_dices_mask = Some(room.change_dices_mask.subscribe());
                data.listen_change_dices = Some(room.change_dices.subscribe());
                return ServerAPIResponse::Register(shared::RegisterResponse(Some(room_id)));
            }
            // The room no longer exists: drop the dangling reference.
            gs.players.get_mut(&uuid).unwrap().room = None;
        }

        ServerAPIResponse::Register(shared::RegisterResponse(None))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_register() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = uuid::Uuid::new_v4();
        let cmd = ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            register_uuid,
            "Alice".to_string(),
        ));
        let mut socket_data = SocketLinkedData::default();

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;

        assert_eq!(
            res,
            ServerAPIResponse::Register(shared::RegisterResponse(None))
        );
        assert_eq!(socket_data.uuid, Some(register_uuid));

        let gs = game_state.lock().await;
        let player = gs.players.get(&register_uuid).unwrap();
        assert_eq!(player.name, "Alice");
        assert!(player.connected);
        std::mem::drop(gs);

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Register(shared::RegisterResponse(None))
        );
    }

    #[tokio::test]
    async fn test_register_version_mismatch() {
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let register_uuid = uuid::Uuid::new_v4();
        let cmd = ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION + 1,
            register_uuid,
            "Alice".to_string(),
        ));
        let mut socket_data = SocketLinkedData::default();

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Error(shared::ErrorResponse::VersionMismatch)
        );
        assert_eq!(socket_data.uuid, None);

        let gs = game_state.lock().await;
        assert!(gs.players.is_empty());
    }

    #[tokio::test]
    async fn test_reregister_reattaches_to_room_and_bumps_epoch() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let uuid = uuid::Uuid::new_v4();

        // First socket: register and create a room
        let mut first = SocketLinkedData::default();
        super::super::test_utils::register(&game_state, &mut first, uuid, "Alice").await;
        let room_uuid = super::super::test_utils::create_room(&game_state, &mut first, "R").await;
        assert!(first.has_all_handler());
        let epoch1 = first.epoch.unwrap();

        // Simulate a disconnect of the first socket (page reload)
        {
            let mut gs = game_state.lock().await;
            super::super::on_socket_disconnect(&mut gs, &first);
            assert!(!gs.players.get(&uuid).unwrap().connected);
        }

        // Second socket (the "reloaded page") registers the same uuid: it is
        // re-attached to the room and the response carries the room id.
        let mut second = SocketLinkedData::default();
        let res = handle_request(
            &ServerAPICommand::Register(shared::RegisterCommand(
                shared::PROTOCOL_VERSION,
                uuid,
                "Alice".to_string(),
            )),
            &game_state,
            &mut second,
        )
        .await;
        assert_eq!(
            res,
            ServerAPIResponse::Register(shared::RegisterResponse(Some(room_uuid)))
        );
        assert!(second.has_all_handler());
        assert!(second.epoch.unwrap() > epoch1);

        let gs = game_state.lock().await;
        let player = gs.players.get(&uuid).unwrap();
        assert!(player.connected);
        assert!(player.disconnected_at.is_none());
        assert_eq!(player.room, Some(room_uuid));
        // The stale first socket can no longer disconnect the player
        drop(gs);
        let mut gs = game_state.lock().await;
        super::super::on_socket_disconnect(&mut gs, &first);
        assert!(gs.players.get(&uuid).unwrap().connected);
    }

    #[tokio::test]
    async fn test_reregister_clears_dangling_room_reference() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let uuid = uuid::Uuid::new_v4();
        let mut socket_data = SocketLinkedData::default();
        let fake_room = super::super::test_utils::register_with_dangling_room(
            &game_state,
            &mut socket_data,
            uuid,
            "Alice",
        )
        .await;

        let cmd = ServerAPICommand::Register(shared::RegisterCommand(
            shared::PROTOCOL_VERSION,
            uuid,
            "Alice".to_string(),
        ));
        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(
            res,
            ServerAPIResponse::Register(shared::RegisterResponse(None))
        );

        let gs = game_state.lock().await;
        assert_eq!(gs.players.get(&uuid).unwrap().room, None);
        assert!(!gs.rooms.contains_key(&fake_room));
    }

}
