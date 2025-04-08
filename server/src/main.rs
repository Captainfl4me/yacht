use futures_util::{SinkExt, StreamExt};
use log::*;
use shared::ServerAPIResponse;
use speedy::{Readable, Writable};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

mod game;
use game::GameState;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("== Start Yatch Server ==");
    let game_state = Arc::new(Mutex::new(GameState::new()));

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

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    game_state: &Arc<Mutex<GameState>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut uuid: Option<uuid::Uuid> = None;
    let party_started_notify: Arc<Notify> = Arc::new(Notify::new());

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if let Message::Binary(msg) = msg {
                            if let Ok(cmd) = shared::ServerAPICommand::read_from_buffer(&msg) {
                                    ws_sender.send(Message::Binary(game::handle_request(&cmd, game_state, &mut uuid, party_started_notify.clone()).await.write_to_vec().unwrap().into())).await?;
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = party_started_notify.notified() => {
                info!("PARTY STARTED");
                ws_sender.send(Message::Binary(ServerAPIResponse::StartGame(shared::StartGameResponse).write_to_vec().unwrap().into())).await?;
            }
        }
    }

    if uuid.is_some() {
        let mut gs = game_state.lock().await;
        if let Some(player) = gs.players.get_mut(&uuid.unwrap()) {
            player.connected = false;
            player.party_start_notify = None;
        }
    }

    Ok(())
}
