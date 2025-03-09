use speedy::{Readable, Writable};
use uuid::Uuid;

#[derive(Debug, Readable, Writable)]
pub struct PingCommand;

#[derive(Debug, Readable, Writable)]
pub struct RegisterCommand(pub Uuid);

#[derive(Debug, Readable, Writable)]
pub struct CreateRoomCommand;

#[derive(Debug, Readable, Writable)]
pub struct ListRoomCommand;

#[derive(Debug, Readable, Writable)]
pub struct JoinRoomCommand;

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
