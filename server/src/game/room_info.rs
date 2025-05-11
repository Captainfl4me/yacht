use std::sync::Arc;

use shared::{
    ErrorResponse, PlayerHeader, RoomInfo, RoomInfoCommand, RoomInfoResponse, ServerAPIResponse,
};
use tokio::sync::Mutex;

use super::Handler;

impl Handler for RoomInfoCommand {
    async fn handle_request(
        &self,
        game_state: &Arc<Mutex<super::GameState>>,
        data: &mut super::super::SocketLinkedData,
    ) -> shared::ServerAPIResponse {
        if let Some(player_uuid) = &data.uuid {
            let gs = game_state.lock().await;
            if gs.players.contains_key(player_uuid) {
                if let Some(room_id) = gs.players.get(player_uuid).unwrap().room {
                    if gs.rooms.contains_key(&room_id) {
                        let players_uuid = gs.rooms.get(&room_id).unwrap().players.clone();

                        ServerAPIResponse::RoomInfo(RoomInfoResponse(RoomInfo {
                            uuid: room_id,
                            name: gs.rooms.get(&room_id).unwrap().header.name.clone(),
                            players: players_uuid
                                .iter()
                                .map(|uuid| PlayerHeader {
                                    uuid: *uuid,
                                    name: "PH".to_string(), // TODO
                                })
                                .collect(),
                            creator: gs.rooms.get(&room_id).unwrap().creator,
                        }))
                    } else {
                        ServerAPIResponse::Error(ErrorResponse::NotFound)
                    }
                } else {
                    ServerAPIResponse::Error(ErrorResponse::NotFound)
                }
            } else {
                ServerAPIResponse::Error(ErrorResponse::NotFound)
            }
        } else {
            ServerAPIResponse::Error(ErrorResponse::NotRegister)
        }
    }
}
