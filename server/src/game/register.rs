use super::{Handler, Player};
use log::info;
use shared::{RegisterCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

impl Handler for RegisterCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        let uuid: Uuid = self.0;
        info!("Receive UUID: {}", uuid);
        data.uuid = Some(uuid);
        let mut gs = game_state.lock().await;
        gs.players
            .entry(uuid)
            .or_insert(Player {
                uuid,
                room: None,
                connected: false,
            })
            .connected = true;

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
        let cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let mut socket_data = SocketLinkedData {
            uuid: None,
            listen_change_game_state: None,
            listen_change_turn: None,
            listen_change_dices_mask: None,
            listen_change_dices: None,
        };

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;

        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        assert_eq!(socket_data.uuid, Some(register_uuid));

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
    }
}
