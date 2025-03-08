use speedy::{Readable, Writable};
use uuid::Uuid;

#[derive(Readable, Writable)]
pub struct PingCommand;

#[derive(Readable, Writable)]
pub struct RegisterCommand(Uuid);

#[derive(Readable, Writable)]
pub struct CreateRoomCommand;

#[derive(Readable, Writable)]
pub struct ListRoomCommand;

#[derive(Readable, Writable)]
pub struct JoinRoomCommand;

#[derive(Readable, Writable)]
pub struct RollCommand;

#[derive(Readable, Writable)]
pub struct KeepDiceCommand;

#[derive(Readable, Writable)]
pub enum ServerAPICommand {
    Ping(PingCommand),
    Register(RegisterCommand),
    CreateRoom(CreateRoomCommand),
    ListRoom(ListRoomCommand),
    JoinRoom(JoinRoomCommand),
    Roll(RollCommand),
    KeepDice(KeepDiceCommand),
}
