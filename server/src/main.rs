use futures_util::{SinkExt, StreamExt};
use log::*;
use shared::{Score, ServerAPIResponse};
use speedy::{Readable, Writable};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};
use uuid::Uuid;

mod game;
use game::GameState;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("== Start Yatch Server ==");
    let game_state = Arc::new(Mutex::new(GameState::new()));

    // The listen address can be overridden (e.g. by the e2e tests, which run
    // the server on a dedicated port to avoid clashing with anything else).
    let addr = std::env::var("YATCH_SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream, game_state.clone()));
    }
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, game_state: Arc<Mutex<GameState>>) {
    if let Err(e) = handle_connection(peer, stream, &game_state).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

#[derive(Default)]
struct SocketLinkedData {
    uuid: Option<Uuid>,
    listen_change_room: Option<broadcast::Receiver<shared::RoomUpdateReason>>,
    listen_change_turn: Option<broadcast::Receiver<usize>>,
    listen_change_score: Option<broadcast::Receiver<(u8, Score)>>,
    listen_change_dices_mask: Option<broadcast::Receiver<[u8; 5]>>,
    listen_change_dices: Option<broadcast::Receiver<[u8; 5]>>,
}
impl SocketLinkedData {
    pub fn has_all_handler(&self) -> bool {
        self.listen_change_room.is_some()
            && self.listen_change_turn.is_some()
            && self.listen_change_score.is_some()
            && self.listen_change_dices_mask.is_some()
            && self.listen_change_dices.is_some()
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    game_state: &Arc<Mutex<GameState>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut socket_data = SocketLinkedData::default();
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        if !socket_data.has_all_handler() {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            // Benign connection errors end the loop so the
                            // disconnect cleanup below still runs; unexpected
                            // errors keep propagating to accept_connection.
                            let msg = match msg {
                                Ok(msg) => msg,
                                Err(Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8) => break,
                                Err(e) => return Err(e),
                            };
                            if let Message::Binary(msg) = msg {
                                if let Ok(cmd) = shared::ServerAPICommand::read_from_buffer(&msg) {
                                    let res = game::handle_request(&cmd, game_state, &mut socket_data).await;
                                    ws_sender.send(Message::Binary(res.write_to_vec().unwrap().into())).await?;
                                }
                            } else if msg.is_close() {
                                break;
                            } else {
                                debug!("Unknow message: {:?}", msg);
                            }
                        }
                        None => break,
                    }
                }
                _ = ping_interval.tick() => {
                    ws_sender.send(Message::Ping(Vec::new().into())).await?;
                }
            };
        } else {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            // Benign connection errors end the loop so the
                            // disconnect cleanup below still runs; unexpected
                            // errors keep propagating to accept_connection.
                            let msg = match msg {
                                Ok(msg) => msg,
                                Err(Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8) => break,
                                Err(e) => return Err(e),
                            };
                            if let Message::Binary(msg) = msg {
                                if let Ok(cmd) = shared::ServerAPICommand::read_from_buffer(&msg) {
                                    let res = game::handle_request(&cmd, game_state, &mut socket_data).await;
                                    ws_sender.send(Message::Binary(res.write_to_vec().unwrap().into())).await?;
                                }
                            } else if msg.is_close() {
                                break;
                            } else {
                                debug!("Unknow message: {:?}", msg);
                            }
                        }
                        None => break,
                    }
                }
                res = socket_data.listen_change_room.as_mut().unwrap().recv() => {
                    match res {
                        Ok(room_update) => {
                            info!("Game state changes");
                            ws_sender.send(Message::Binary(ServerAPIResponse::RoomUpdate(shared::RoomUpdateResponse(room_update)).write_to_vec().unwrap().into())).await?;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Room update channel lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            socket_data.listen_change_room = None;
                        }
                    }
                }
                res = socket_data.listen_change_turn.as_mut().unwrap().recv() => {
                    match res {
                        Ok(new_turn) => {
                            info!("Turn changes");
                            ws_sender.send(Message::Binary(ServerAPIResponse::ChangeTurn(shared::ChangeTurnResponse(new_turn as u8)).write_to_vec().unwrap().into())).await?;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Turn channel lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            socket_data.listen_change_turn = None;
                        }
                    }
                }
                res = socket_data.listen_change_score.as_mut().unwrap().recv() => {
                    match res {
                        Ok((player, new_score)) => {
                            info!("Score changes");
                            ws_sender.send(Message::Binary(ServerAPIResponse::ChangeScore(shared::ChangeScoreResponse(player, new_score)).write_to_vec().unwrap().into())).await?;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Score channel lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            socket_data.listen_change_score = None;
                        }
                    }
                }
                res = socket_data.listen_change_dices_mask.as_mut().unwrap().recv() => {
                    match res {
                        Ok(dice_mask) => {
                            info!("Dices mask changes");
                            ws_sender.send(Message::Binary(ServerAPIResponse::KeepDice(shared::KeepDiceResponse(dice_mask)).write_to_vec().unwrap().into())).await?;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Dices mask channel lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            socket_data.listen_change_dices_mask = None;
                        }
                    }
                }
                res = socket_data.listen_change_dices.as_mut().unwrap().recv() => {
                    match res {
                        Ok(dices) => {
                            info!("Dices changes");
                            ws_sender.send(Message::Binary(ServerAPIResponse::Roll(shared::RollResponse(dices)).write_to_vec().unwrap().into())).await?;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("Dices channel lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            socket_data.listen_change_dices = None;
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    ws_sender.send(Message::Ping(Vec::new().into())).await?;
                }
            };
        }
    }

    if let Some(uuid) = socket_data.uuid {
        info!("Close player connection: {}", uuid);
        let mut gs = game_state.lock().await;
        if let Some(player) = gs.players.get_mut(&uuid) {
            player.connected = false;
        }
        let room_id = gs.players.get(&uuid).and_then(|player| player.room);
        if let Some(room_id) = room_id {
            game::remove_player_from_room(&mut gs, &room_id, &uuid);
        }
    }

    Ok(())
}
