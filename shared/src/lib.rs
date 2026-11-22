use speedy::{Readable, Writable};

mod command;
mod response;

pub use command::*;
pub use response::*;
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 1;

pub const MAX_PLAYERS_PER_ROOM: usize = 8;

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
}
