use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;

#[derive(Debug, Readable, Writable, Clone, PartialEq, Eq)]
pub struct RoomHeader {
    pub uuid: uuid::Uuid,
    pub name: String
}

#[derive(Debug, Readable, Writable, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    NotStarted,
    Started,
    Scoreboard,
}
