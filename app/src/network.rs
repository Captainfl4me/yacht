use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use macroquad::prelude::{error, info};

pub enum ServerAPICommand {
    Ping,
    CreateRoom,
    ListRoom,
    JoinRoom,
    Roll,
    KeepDice,
}

#[derive(PartialEq, Eq)]
pub enum NetworkManagerState {
    NotInit,
    Connecting,
    Connected,
}

const MAX_RETRY: u8 = 5;

pub struct NetworkManager<'a> {
    state: NetworkManagerState,
    backend_url: &'a str,
    ws_sender: Option<WsSender>,
    ws_receiver: Option<WsReceiver>,
    error_connection_retry: u8,
}

impl<'a> NetworkManager<'a> {
    pub fn new(backend_url: &'a str) -> Self {
        NetworkManager {
            state: NetworkManagerState::NotInit,
            backend_url,
            ws_sender: None,
            ws_receiver: None,
            error_connection_retry: 0,
        }
    }

    pub fn udpate(&mut self) {
        match self.state {
            NetworkManagerState::NotInit => {
                if self.error_connection_retry < MAX_RETRY {
                    if self.error_connection_retry > 0 {
                        info!("Retrying connection: {}", self.error_connection_retry);
                    }
                    let (sender, receiver) =
                        ewebsock::connect(self.backend_url, ewebsock::Options::default()).unwrap();
                    self.ws_sender = Some(sender);
                    self.ws_receiver = Some(receiver);
                    self.state = NetworkManagerState::Connecting;
                }
            }
            NetworkManagerState::Connecting => {
                if let Some(event) = self.ws_receiver.as_ref().unwrap().try_recv() {
                    match event {
                        WsEvent::Opened => self.state = NetworkManagerState::Connected,
                        WsEvent::Error(error_str) => {
                            error!("{}", error_str);
                            self.state = NetworkManagerState::NotInit;
                            self.error_connection_retry += 1;
                        }
                        _ => (),
                    }
                }
            }
            NetworkManagerState::Connected => {
                if let Some(event) = self.ws_receiver.as_ref().unwrap().try_recv() {
                    match event {
                        WsEvent::Message(msg) => info!("message!"),
                        _ => (),
                    }
                }
                // let val = futures_util::future::poll_fn(|ctx| {
                //     match self.ws.poll_next_unpin(ctx) {
                //         std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                //         std::task::Poll::Ready(Some(msg)) => std::task::Poll::Ready(Some(msg)),
                //         std::task::Poll::Pending => std::task::Poll::Ready(None)
                //     }
                // }).await;
                //
                // if val.is_some() {
                //     println!("Recv");
                // }
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.state == NetworkManagerState::Connected
    }

    pub fn send_command(&mut self, command: ServerAPICommand) {
        if self.is_connected() {
            if let Some(ws) = self.ws_sender.as_mut() {
                let data: Vec<u8> = match command {
                    ServerAPICommand::Ping => vec![0u8],
                    ServerAPICommand::CreateRoom => vec![1u8],
                    ServerAPICommand::ListRoom => vec![2u8],
                    ServerAPICommand::JoinRoom => vec![3u8],
                    ServerAPICommand::Roll => vec![4u8],
                    ServerAPICommand::KeepDice => vec![5u8],
                };

                ws.send(WsMessage::Binary(data));
            }
        }
    }
}
