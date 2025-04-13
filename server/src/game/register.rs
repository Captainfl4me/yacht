use super::{Handler, Player};
use log::info;
use shared::{RegisterCommand, ServerAPIResponse};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

impl Handler for RegisterCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        player_uuid: &mut Option<uuid::Uuid>,
        party_start_notify: Arc<Notify>,
    ) -> shared::ServerAPIResponse {
        let uuid: Uuid = self.0;
        info!("Receive UUID: {}", uuid);
        *player_uuid = Some(uuid);
        let mut gs = game_state.lock().await;
        gs.players
            .entry(uuid)
            .or_insert(Player {
                uuid,
                room: None,
                connected: false,
                party_start_notify: Some(party_start_notify),
            })
            .connected = true;

        ServerAPIResponse::Register(shared::RegisterResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{handle_request, GameState};
    use super::*;
    use shared::ServerAPICommand;

    #[tokio::test]
    async fn test_register() {
        let game_state = Arc::new(Mutex::new(GameState::new()));
        let mut uuid: Option<uuid::Uuid> = None;

        let register_uuid = uuid::Uuid::new_v4();
        let cmd = ServerAPICommand::Register(shared::RegisterCommand(register_uuid));
        let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

        let res = handle_request(&cmd, &game_state, &mut uuid, party_started_notify.clone()).await;

        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
        assert_eq!(uuid, Some(register_uuid));

        let res = handle_request(&cmd, &game_state, &mut uuid, party_started_notify.clone()).await;
        assert_eq!(res, ServerAPIResponse::Register(shared::RegisterResponse));
    }
}
