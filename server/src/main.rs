use futures_util::{SinkExt, StreamExt};
use log::*;
use shared::ServerAPIResponse;
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

struct SocketLinkedData {
    uuid: Option<Uuid>,
    listen_change_game_state: Option<broadcast::Receiver<shared::GameState>>,
    listen_change_turn: Option<broadcast::Receiver<usize>>,
    listen_change_dices_mask: Option<broadcast::Receiver<[u8; 5]>>,
    listen_change_dices: Option<broadcast::Receiver<[u8; 5]>>,
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    game_state: &Arc<Mutex<GameState>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut socket_data = SocketLinkedData {
        uuid: None,
        listen_change_game_state: None,
        listen_change_turn: None,
        listen_change_dices_mask: None,
        listen_change_dices: None,
    };

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        let has_handler_listen_change_game_state = socket_data.listen_change_game_state.as_ref().is_some();
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
                        }
                    }
                    None => break,
                }
            }
            res = socket_data.listen_change_game_state.as_mut().unwrap().recv(), if has_handler_listen_change_game_state => {
                if let Ok(game_state) = res {
                    info!("Game state changes");
                    ws_sender.send(Message::Binary(ServerAPIResponse::GameState(shared::GameStateResponse(game_state)).write_to_vec().unwrap().into())).await?;
                } else {
                    error!("Error: {:?}", res);
                }
            }
        }
    }

    if socket_data.uuid.is_some() {
        let mut gs = game_state.lock().await;
        if let Some(player) = gs.players.get_mut(&socket_data.uuid.unwrap()) {
            player.connected = false;
        }
    }

    Ok(())
}
