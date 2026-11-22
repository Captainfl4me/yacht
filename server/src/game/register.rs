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
        data.uuid = Some(uuid);
        let mut gs = game_state.lock().await;
        let player = gs.players.entry(uuid).or_insert(Player {
            uuid,
            name: name.clone(),
            room: None,
            connected: false,
        });
        player.connected = true;
        player.name = name;

        ServerAPIResponse::Register(shared::RegisterResponse)
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

        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        assert_eq!(socket_data.uuid, Some(register_uuid));

        let gs = game_state.lock().await;
        let player = gs.players.get(&register_uuid).unwrap();
        assert_eq!(player.name, "Alice");
        assert!(player.connected);
        std::mem::drop(gs);

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
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
}
