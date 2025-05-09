use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;
use uuid::Uuid;

#[derive(Debug, Readable, Writable, Clone, PartialEq, Eq)]
pub struct PlayerHeader {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Readable, Writable, Clone, PartialEq, Eq)]
pub struct RoomHeader {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Readable, Writable, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub uuid: Uuid,
    pub name: String,
    pub players: Vec<PlayerHeader>,
    pub creator: Uuid,
}

#[derive(Debug, Readable, Writable, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    NotStarted,
    Started,
    Scoreboard,
}

#[derive(Debug, Readable, Writable, PartialEq, Eq, Clone, Copy)]
pub enum Score {
    Aces(u8),
    Twos(u8),
    Threes(u8),
    Fours(u8),
    Fives(u8),
    Sixes(u8),
    ThreeOfAKind(u8),
    FourOfAKind(u8),
    Fullhouse(u8),
    SmallStraight(u8),
    LargeStraight(u8),
    Yacht(u8),
    Chance(u8),
}
