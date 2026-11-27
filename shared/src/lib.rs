use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 1;

pub const MAX_PLAYERS_PER_ROOM: usize = 8;

pub const MAX_THROWS_PER_TURN: u8 = 3;

pub const UPPER_SECTION_BONUS_THRESHOLD: u16 = 63;
pub const UPPER_SECTION_BONUS: u16 = 35;
pub const YACHT_BONUS: u16 = 50;

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

#[derive(Debug, Readable, Writable, Clone, Default, PartialEq, Eq)]
pub struct ScoreSheet {
    pub aces: Option<u8>,
    pub twos: Option<u8>,
    pub threes: Option<u8>,
    pub fours: Option<u8>,
    pub fives: Option<u8>,
    pub sixes: Option<u8>,
    pub three_of_a_kind: Option<u8>,
    pub four_of_a_kind: Option<u8>,
    pub fullhouse: Option<u8>,
    pub small_straight: Option<u8>,
    pub large_straight: Option<u8>,
    pub yacht: Option<u8>,
    pub chance: Option<u8>,
    pub extra_yachts: u8,
}

impl ScoreSheet {
    pub fn set(&mut self, score: Score) {
        match score {
            Score::Aces(v) => self.aces = Some(v),
            Score::Twos(v) => self.twos = Some(v),
            Score::Threes(v) => self.threes = Some(v),
            Score::Fours(v) => self.fours = Some(v),
            Score::Fives(v) => self.fives = Some(v),
            Score::Sixes(v) => self.sixes = Some(v),
            Score::ThreeOfAKind(v) => self.three_of_a_kind = Some(v),
            Score::FourOfAKind(v) => self.four_of_a_kind = Some(v),
            Score::Fullhouse(v) => self.fullhouse = Some(v),
            Score::SmallStraight(v) => self.small_straight = Some(v),
            Score::LargeStraight(v) => self.large_straight = Some(v),
            Score::Yacht(v) => self.yacht = Some(v),
            Score::Chance(v) => self.chance = Some(v),
        }
    }

    pub fn is_full(&self) -> bool {
        self.aces.is_some()
            && self.twos.is_some()
            && self.threes.is_some()
            && self.fours.is_some()
            && self.fives.is_some()
            && self.sixes.is_some()
            && self.three_of_a_kind.is_some()
            && self.four_of_a_kind.is_some()
            && self.fullhouse.is_some()
            && self.small_straight.is_some()
            && self.large_straight.is_some()
            && self.yacht.is_some()
            && self.chance.is_some()
    }

    pub fn upper_subtotal(&self) -> u16 {
        [
            self.aces,
            self.twos,
            self.threes,
            self.fours,
            self.fives,
            self.sixes,
        ]
        .into_iter()
        .map(|s| s.unwrap_or(0) as u16)
        .sum()
    }

    pub fn upper_bonus(&self) -> u16 {
        if self.upper_subtotal() >= UPPER_SECTION_BONUS_THRESHOLD {
            UPPER_SECTION_BONUS
        } else {
            0
        }
    }

    pub fn total(&self) -> u16 {
        let sum: u16 = [
            self.aces,
            self.twos,
            self.threes,
            self.fours,
            self.fives,
            self.sixes,
            self.three_of_a_kind,
            self.four_of_a_kind,
            self.fullhouse,
            self.small_straight,
            self.large_straight,
            self.yacht,
            self.chance,
        ]
        .into_iter()
        .map(|s| s.unwrap_or(0) as u16)
        .sum();
        sum + self.upper_bonus() + YACHT_BONUS * self.extra_yachts as u16
    }
}

pub fn count_points_for_numbers(dices: &[u8; 5], number: u8) -> u8 {
    let mut points = 0;
    for dice in dices.iter() {
        if *dice == number {
            points += *dice;
        }
    }

    points
}

pub fn count_points_for_identical(dices: &[u8; 5], number: u8) -> u8 {
    let mut value_freq = [0; 6];

    for dice in dices {
        value_freq[(*dice - 1) as usize] += 1;
    }

    if *value_freq.iter().max().unwrap() >= number {
        if number == 5 {
            50
        } else {
            dices.iter().sum()
        }
    } else {
        0
    }
}

pub fn count_points_for_fullhouse(dices: &[u8; 5]) -> u8 {
    let mut value_freq = [0; 6];

    for dice in dices {
        value_freq[(*dice - 1) as usize] += 1;
    }

    let freq_max = *value_freq.iter().max().unwrap();
    let freq_min = *value_freq.iter().filter(|x| **x > 0).min().unwrap();

    if freq_max == 5 || (freq_max == 3 && freq_min == 2) {
        25
    } else {
        0
    }
}

pub fn count_points_for_straight(dices: &[u8; 5], size: u8) -> u8 {
    let mut value_straight_size = [0; 5];
    let mut dices_sorted = Vec::from(*dices);
    dices_sorted.sort();
    dices_sorted.dedup();

    for (index, dice) in dices_sorted.iter().enumerate() {
        let mut prev_val = *dice;
        let mut counter = 1;
        for next_dice_sort in dices_sorted.iter().skip(index + 1) {
            if (prev_val + 1) != *next_dice_sort {
                break;
            }
            prev_val = *next_dice_sort;
            counter += 1;
        }

        value_straight_size[index] = counter;
    }

    if *value_straight_size.iter().max().unwrap() >= size {
        if size == 5 {
            40
        } else {
            30
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_counter() {
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 1), 1);
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 2), 4);
        assert_eq!(count_points_for_numbers(&[1, 2, 5, 6, 2], 3), 0);
        assert_eq!(count_points_for_numbers(&[4, 2, 4, 4, 4], 4), 16);
        assert_eq!(count_points_for_numbers(&[5, 5, 5, 5, 5], 5), 25);
        assert_eq!(count_points_for_numbers(&[6, 6, 5, 6, 2], 6), 18);
    }

    #[test]
    fn test_points_for_identical() {
        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 1), 16);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 3), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 3), 25);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 1], 3), 21);
        assert_eq!(count_points_for_identical(&[4, 4, 4, 6, 1], 3), 19);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 4), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 4), 25);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 1], 4), 21);

        assert_eq!(count_points_for_identical(&[1, 2, 5, 6, 2], 5), 0);
        assert_eq!(count_points_for_identical(&[5, 5, 5, 5, 5], 5), 50);
    }

    #[test]
    fn test_points_for_fullhouse() {
        assert_eq!(count_points_for_fullhouse(&[2, 2, 6, 6, 6]), 25);
        assert_eq!(count_points_for_fullhouse(&[1, 1, 1, 1, 1]), 25);
        assert_eq!(count_points_for_fullhouse(&[5, 5, 5, 5, 1]), 0);
        assert_eq!(count_points_for_fullhouse(&[1, 2, 5, 6, 2]), 0);
    }

    #[test]
    fn test_points_for_straight() {
        assert_eq!(count_points_for_straight(&[2, 3, 4, 5, 6], 4), 30);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 5, 6], 4), 30);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 3, 6], 4), 0);
        assert_eq!(count_points_for_straight(&[5, 4, 3, 6, 4], 4), 30);

        assert_eq!(count_points_for_straight(&[2, 3, 4, 5, 6], 5), 40);
        assert_eq!(count_points_for_straight(&[1, 3, 4, 5, 6], 5), 0);
    }

    #[test]
    fn test_score_sheet_upper_bonus_and_total() {
        let sheet = ScoreSheet::default();
        assert!(!sheet.is_full());
        assert_eq!(sheet.upper_subtotal(), 0);
        assert_eq!(sheet.upper_bonus(), 0);

        let sheet = ScoreSheet {
            aces: Some(3),
            twos: Some(6),
            threes: Some(9),
            fours: Some(12),
            fives: Some(25),
            sixes: Some(18),
            chance: Some(15),
            ..Default::default()
        };

        assert_eq!(sheet.upper_subtotal(), 73);
        assert_eq!(sheet.upper_bonus(), 35);
        assert_eq!(sheet.total(), 73 + 35 + 15);

        let sheet = ScoreSheet {
            extra_yachts: 2,
            ..sheet
        };
        assert_eq!(sheet.total(), 73 + 35 + 15 + 2 * 50);

        let sheet = ScoreSheet {
            three_of_a_kind: Some(0),
            four_of_a_kind: Some(0),
            fullhouse: Some(0),
            small_straight: Some(0),
            large_straight: Some(0),
            yacht: Some(0),
            ..sheet
        };
        assert!(sheet.is_full());
    }

    #[test]
    fn test_score_sheet_set() {
        let mut sheet = ScoreSheet::default();
        sheet.set(Score::Aces(3));
        sheet.set(Score::Yacht(50));
        sheet.set(Score::Chance(15));
        assert_eq!(sheet.aces, Some(3));
        assert_eq!(sheet.yacht, Some(50));
        assert!(!sheet.is_full());
        assert_eq!(sheet.total(), 3 + 50 + 15);
    }

    #[test]
    fn test_score_sheet_no_upper_bonus_below_threshold() {
        let sheet = ScoreSheet {
            aces: Some(1),
            twos: Some(2),
            threes: Some(3),
            fours: Some(4),
            fives: Some(5),
            sixes: Some(6),
            ..Default::default()
        };
        assert_eq!(sheet.upper_subtotal(), 21);
        assert_eq!(sheet.upper_bonus(), 0);
        assert_eq!(sheet.total(), 21);
    }

    #[test]
    fn test_score_sheet_set_all_variants() {
        let mut sheet = ScoreSheet::default();
        sheet.set(Score::Aces(1));
        sheet.set(Score::Twos(2));
        sheet.set(Score::Threes(3));
        sheet.set(Score::Fours(4));
        sheet.set(Score::Fives(5));
        sheet.set(Score::Sixes(6));
        sheet.set(Score::ThreeOfAKind(7));
        sheet.set(Score::FourOfAKind(8));
        sheet.set(Score::Fullhouse(9));
        sheet.set(Score::SmallStraight(10));
        sheet.set(Score::LargeStraight(11));
        sheet.set(Score::Yacht(12));
        sheet.set(Score::Chance(13));
        assert_eq!(sheet.aces, Some(1));
        assert_eq!(sheet.twos, Some(2));
        assert_eq!(sheet.threes, Some(3));
        assert_eq!(sheet.fours, Some(4));
        assert_eq!(sheet.fives, Some(5));
        assert_eq!(sheet.sixes, Some(6));
        assert_eq!(sheet.three_of_a_kind, Some(7));
        assert_eq!(sheet.four_of_a_kind, Some(8));
        assert_eq!(sheet.fullhouse, Some(9));
        assert_eq!(sheet.small_straight, Some(10));
        assert_eq!(sheet.large_straight, Some(11));
        assert_eq!(sheet.yacht, Some(12));
        assert_eq!(sheet.chance, Some(13));
        assert!(sheet.is_full());
    }

    #[test]
    fn test_points_for_identical_four_of_a_kind_sums_all_dices() {
        assert_eq!(count_points_for_identical(&[6, 6, 6, 6, 2], 4), 26);
        assert_eq!(count_points_for_identical(&[1, 1, 2, 2, 2], 3), 8);
        // five of a kind also satisfies four of a kind
        assert_eq!(count_points_for_identical(&[3, 3, 3, 3, 3], 4), 15);
    }

    #[test]
    fn test_points_for_straight_edge_cases() {
        // large straight starting at 1
        assert_eq!(count_points_for_straight(&[1, 2, 3, 4, 5], 5), 40);
        assert_eq!(count_points_for_straight(&[5, 4, 3, 2, 1], 5), 40);
        // not a large straight
        assert_eq!(count_points_for_straight(&[1, 2, 3, 4, 6], 5), 0);
        // small straight with duplicated dice
        assert_eq!(count_points_for_straight(&[1, 2, 3, 4, 4], 4), 30);
        assert_eq!(count_points_for_straight(&[1, 2, 2, 3, 4], 4), 30);
        assert_eq!(count_points_for_straight(&[3, 3, 4, 5, 6], 4), 30);
        // longest run is only 3
        assert_eq!(count_points_for_straight(&[1, 2, 4, 5, 6], 4), 0);
        assert_eq!(count_points_for_straight(&[1, 3, 5, 6, 2], 4), 0);
    }

    #[test]
    fn test_upper_bonus_exact_threshold() {
        // 8 + 25 + 30 = 63 -> exactly at the threshold, bonus applies
        let sheet = ScoreSheet {
            fours: Some(8),
            fives: Some(25),
            sixes: Some(30),
            ..Default::default()
        };
        assert_eq!(sheet.upper_subtotal(), UPPER_SECTION_BONUS_THRESHOLD);
        assert_eq!(sheet.upper_bonus(), UPPER_SECTION_BONUS);
        assert_eq!(sheet.total(), UPPER_SECTION_BONUS_THRESHOLD + UPPER_SECTION_BONUS);

        // 62 -> one below the threshold, no bonus
        let sheet = ScoreSheet {
            fours: Some(7),
            fives: Some(25),
            sixes: Some(30),
            ..Default::default()
        };
        assert_eq!(sheet.upper_subtotal(), 62);
        assert_eq!(sheet.upper_bonus(), 0);
        assert_eq!(sheet.total(), 62);
    }

    #[test]
    fn test_command_serialization_round_trip() {
        let uuid = Uuid::parse_str("123e4567-e89b-42d3-a456-426614174000").unwrap();
        let commands = vec![
            ServerAPICommand::Ping(PingCommand),
            ServerAPICommand::Register(RegisterCommand(
                PROTOCOL_VERSION,
                uuid,
                "Alice".to_string(),
            )),
            ServerAPICommand::CreateRoom(CreateRoomCommand("room".to_string())),
            ServerAPICommand::ListRoom(ListRoomCommand),
            ServerAPICommand::JoinRoom(JoinRoomCommand(uuid)),
            ServerAPICommand::StartGame(StartGameCommand),
            ServerAPICommand::Roll(RollCommand),
            ServerAPICommand::KeepDice(KeepDiceCommand([1, 0, 1, 0, 1])),
            ServerAPICommand::SelectPoints(SelectPointsCommand(Score::Yacht(50))),
            ServerAPICommand::RoomInfo(RoomInfoCommand),
            ServerAPICommand::LeaveRoom(LeaveRoomCommand),
            ServerAPICommand::RestartGame(RestartGameCommand),
        ];
        for cmd in commands {
            let bytes = cmd.write_to_vec().unwrap();
            let decoded = ServerAPICommand::read_from_buffer(&bytes).unwrap();
            assert_eq!(cmd, decoded);
        }
    }

    #[test]
    fn test_response_serialization_round_trip() {
        let uuid = Uuid::parse_str("123e4567-e89b-42d3-a456-426614174000").unwrap();
        let player = PlayerHeader {
            uuid,
            name: "Alice".to_string(),
        };
        let responses = vec![
            ServerAPIResponse::Ok,
            ServerAPIResponse::Ping(PingResponse),
            ServerAPIResponse::Register(RegisterResponse),
            ServerAPIResponse::CreateRoom(CreateRoomResponse(uuid)),
            ServerAPIResponse::ListRoom(ListRoomResponse(vec![RoomHeader {
                uuid,
                name: "r".to_string(),
            }])),
            ServerAPIResponse::JoinRoom(JoinRoomResponse(uuid)),
            ServerAPIResponse::Roll(RollResponse([1, 2, 3, 4, 5])),
            ServerAPIResponse::KeepDice(KeepDiceResponse([1, 0, 0, 1, 0])),
            ServerAPIResponse::ChangeTurn(ChangeTurnResponse(2)),
            ServerAPIResponse::ChangeScore(ChangeScoreResponse(1, Score::Fullhouse(25))),
            ServerAPIResponse::RoomUpdate(RoomUpdateResponse(RoomUpdateReason::NewPlayer(
                player.clone(),
            ))),
            ServerAPIResponse::RoomUpdate(RoomUpdateResponse(
                RoomUpdateReason::GameStateChange(GameState::Scoreboard),
            )),
            ServerAPIResponse::RoomUpdate(RoomUpdateResponse(RoomUpdateReason::FinalScores(
                vec![FinalScoreEntry {
                    player: player.clone(),
                    total: 123,
                }],
            ))),
            ServerAPIResponse::Error(ErrorResponse::NotRegister),
            ServerAPIResponse::Error(ErrorResponse::NotFound),
            ServerAPIResponse::Error(ErrorResponse::PlayerAlreadyInRoom),
            ServerAPIResponse::Error(ErrorResponse::NotInRoom),
            ServerAPIResponse::Error(ErrorResponse::NotEnoughPermission),
            ServerAPIResponse::Error(ErrorResponse::VersionMismatch),
            ServerAPIResponse::Error(ErrorResponse::RoomFull),
            ServerAPIResponse::Error(ErrorResponse::GameAlreadyStarted),
            ServerAPIResponse::RoomInfo(RoomInfoResponse(RoomInfo {
                uuid,
                name: "r".to_string(),
                players: vec![player.clone()],
                creator: uuid,
            })),
            ServerAPIResponse::LeaveRoom(LeaveRoomResponse),
        ];
        for res in responses {
            let bytes = res.write_to_vec().unwrap();
            let decoded = ServerAPIResponse::read_from_buffer(&bytes).unwrap();
            assert_eq!(res, decoded);
        }
    }
}
