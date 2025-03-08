use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;

#[derive(Readable, Writable)]
pub struct RoomHeader {
    id: u64,
    name: String
}
