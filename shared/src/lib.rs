use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;

#[derive(Debug, Readable, Writable, Clone)]
pub struct RoomHeader {
    pub uuid: uuid::Uuid,
    pub name: String
}
