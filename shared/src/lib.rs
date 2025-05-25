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
}
