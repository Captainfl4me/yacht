use super::Handler;
use log::info;
use shared::{PingCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

impl Handler for PingCommand {
    async fn handle_request(
        &self,
        _game_state: &Arc<Mutex<super::GameState>>,
        _data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        info!("Receive Ping");
        ServerAPIResponse::Ping(shared::PingResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use crate::SocketLinkedData;
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_ping() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let cmd = ServerAPICommand::Ping(shared::PingCommand);
        let mut socket_data = SocketLinkedData {
            uuid: None,
            listen_change_game_state: None,
            listen_change_turn: None,
            listen_change_dices_mask: None,
            listen_change_dices: None,
        };

        let res = handle_request(&cmd, &game_state, &mut socket_data).await;

        assert_eq!(res, ServerAPIResponse::Ping(shared::PingResponse));
    }
}
