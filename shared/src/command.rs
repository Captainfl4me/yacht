use speedy::{Readable, Writable};
use uuid::Uuid;

#[derive(Debug, Readable, Writable)]
pub struct PingCommand;

#[derive(Debug, Readable, Writable)]
pub struct RegisterCommand(pub Uuid);

#[derive(Debug, Readable, Writable)]
pub struct CreateRoomCommand(pub String);

#[derive(Debug, Readable, Writable)]
pub struct ListRoomCommand;

#[derive(Debug, Readable, Writable)]
pub struct JoinRoomCommand(pub Uuid);

#[derive(Debug, Readable, Writable)]
pub struct RollCommand;

#[derive(Debug, Readable, Writable)]
pub struct KeepDiceCommand;

#[derive(Debug, Readable, Writable)]
pub enum ServerAPICommand {
    Ping(PingCommand),
    Register(RegisterCommand),
    CreateRoom(CreateRoomCommand),
    ListRoom(ListRoomCommand),
    JoinRoom(JoinRoomCommand),
    Roll(RollCommand),
    KeepDice(KeepDiceCommand),
}
