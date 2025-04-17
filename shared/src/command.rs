use speedy::{Readable, Writable};
use uuid::Uuid;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct PingCommand;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RegisterCommand(pub Uuid);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct CreateRoomCommand(pub String);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct ListRoomCommand;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct JoinRoomCommand(pub Uuid);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct StartGameCommand;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct RollCommand;

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub struct KeepDiceCommand(pub [u8; 5]);

#[derive(Debug, Readable, Writable, PartialEq, Eq)]
pub enum ServerAPICommand {
    Ping(PingCommand),
    Register(RegisterCommand),
    CreateRoom(CreateRoomCommand),
    ListRoom(ListRoomCommand),
    JoinRoom(JoinRoomCommand),
    StartGame(StartGameCommand),
    Roll(RollCommand),
    KeepDice(KeepDiceCommand),
}
