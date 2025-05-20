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

    // TEST data
    {
        let mut gs = game_state.lock().await;
        let test_room = game::Room::new(uuid::Uuid::new_v4(), "Coucou".to_string());
        gs.rooms.insert(test_room.uuid(), test_room);
    }

    let addr = "0.0.0.0:8080";
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
    listen_change_game_state: Option<broadcast::Receiver<shared::GameState>>,
    listen_change_turn: Option<broadcast::Receiver<usize>>,
    listen_change_score: Option<broadcast::Receiver<(u8, Score)>>,
    listen_change_dices_mask: Option<broadcast::Receiver<[u8; 5]>>,
    listen_change_dices: Option<broadcast::Receiver<[u8; 5]>>,
}
impl SocketLinkedData {
    pub fn has_all_handler(&self) -> bool {
        self.listen_change_game_state.is_some()
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

    loop {
        if !socket_data.has_all_handler() {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            let msg = msg?;
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
            };
        } else {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            let msg = msg?;
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
                res = socket_data.listen_change_game_state.as_mut().unwrap().recv() => {
                    if let Ok(game_state) = res {
                        info!("Game state changes");
                        ws_sender.send(Message::Binary(ServerAPIResponse::GameState(shared::GameStateResponse(game_state)).write_to_vec().unwrap().into())).await?;
                    } else {
                        error!("Error: {:?}", res);
                    }
                }
                res = socket_data.listen_change_turn.as_mut().unwrap().recv() => {
                    if let Ok(new_turn) = res {
                        info!("Turn changes");
                        ws_sender.send(Message::Binary(ServerAPIResponse::ChangeTurn(shared::ChangeTurnResponse(new_turn as u8)).write_to_vec().unwrap().into())).await?;
                    } else {
                        error!("Error: {:?}", res);
                    }
                }
                res = socket_data.listen_change_score.as_mut().unwrap().recv() => {
                    if let Ok((player, new_score)) = res {
                        info!("Score changes");
                        ws_sender.send(Message::Binary(ServerAPIResponse::ChangeScore(shared::ChangeScoreResponse(player, new_score)).write_to_vec().unwrap().into())).await?;
                    } else {
                        error!("Error: {:?}", res);
                    }
                }
                res = socket_data.listen_change_dices_mask.as_mut().unwrap().recv() => {
                    if let Ok(dice_mask) = res {
                        info!("Dices mask changes");
                        ws_sender.send(Message::Binary(ServerAPIResponse::KeepDice(shared::KeepDiceResponse(dice_mask)).write_to_vec().unwrap().into())).await?;
                    } else {
                        error!("Error: {:?}", res);
                    }
                }
                res = socket_data.listen_change_dices.as_mut().unwrap().recv() => {
                    if let Ok(dices) = res {
                        info!("Dices changes");
                        ws_sender.send(Message::Binary(ServerAPIResponse::Roll(shared::RollResponse(dices)).write_to_vec().unwrap().into())).await?;
                    } else {
                        error!("Error: {:?}", res);
                    }
                }
            };
        }
    }

    if socket_data.uuid.is_some() {
        info!("Close player connection: {}", socket_data.uuid.unwrap());
        let mut room_id = None;
        let mut gs = game_state.lock().await;
        if let Some(player) = gs.players.get_mut(&socket_data.uuid.unwrap()) {
            player.connected = false;
            room_id = player.room;
            player.room = None;
        }

        if let Some(room_id) = room_id {
            if gs.rooms.get_mut(&room_id).unwrap().players.len() <= 1 {
                gs.rooms.remove(&room_id);
            } else {
                gs.rooms
                    .get_mut(&room_id)
                    .unwrap()
                    .players
                    .retain(|uuid| *uuid != socket_data.uuid.unwrap());
            }
        }
    }

    Ok(())
}
