use super::Handler;
use log::info;
use shared::{PingCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

impl Handler for PingCommand {
    async fn handle_request(
        &self,
        _game_state: &Arc<Mutex<super::GameState>>,
        _player_uuid: &mut Option<uuid::Uuid>,
        _party_start_notify: Arc<Notify>,
    ) -> shared::ServerAPIResponse {
        info!("Receive Ping");
        ServerAPIResponse::Ping(shared::PingResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_ping() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;
        let cmd = ServerAPICommand::Ping(shared::PingCommand);
        let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

        let res = handle_request(&cmd, &game_state, &mut uuid, party_started_notify.clone()).await;

        assert_eq!(res, ServerAPIResponse::Ping(shared::PingResponse));
    }
}
