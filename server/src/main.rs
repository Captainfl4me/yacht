use futures_util::{SinkExt, StreamExt};
use log::*;
use speedy::Writable;
use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};
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
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    let mut has_register = false;
    let mut uuid: Option<uuid::Uuid> = None;

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if let Message::Binary(msg) = msg {
                            if let Some(res) = game::handle_request(&msg, game_state, &mut has_register, &mut uuid).await {
                                ws_sender.send(Message::Binary(res.write_to_vec().unwrap().into())).await?;
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                //ws_sender.send(Message::text("tick")).await?;
            }
        }
    }

    if uuid.is_some() {
        let mut gs = game_state.lock().await;
        if let Some(player) = gs.players.get_mut(&uuid.unwrap()) {
            player.connected = false;
        }
    }

    Ok(())
}
