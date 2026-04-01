use std::convert::TryFrom;

use crate::errors::CreateKeyError;

/// A struct representing melodic key of the track
#[derive(Copy, Clone, Debug)]
pub struct Key {
    /// num (1..=12) of the key
    num: i8,
    /// letter (A or B) of the key
    letter: char,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_str: String = (*self).into();
        write!(f, "{}", key_str)
    }
}

impl From<Key> for String {
    fn from(value: Key) -> Self {
        format!("{}{}", value.num, value.letter)
    }
}

impl TryFrom<&str> for Key {
    type Error = CreateKeyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "8B" => Key::new(8, 'B'),
            "3B" => Key::new(3, 'B'),
            "10B" => Key::new(10, 'B'),
            "5B" => Key::new(5, 'B'),
            "12B" => Key::new(12, 'B'),
            "7B" => Key::new(7, 'B'),
            "2B" => Key::new(2, 'B'),
            "9B" => Key::new(9, 'B'),
            "4B" => Key::new(4, 'B'),
            "11B" => Key::new(11, 'B'),
            "6B" => Key::new(6, 'B'),
            "1B" => Key::new(1, 'B'),
            "5A" => Key::new(5, 'A'),
            "12A" => Key::new(12, 'A'),
            "7A" => Key::new(7, 'A'),
            "2A" => Key::new(2, 'A'),
            "9A" => Key::new(9, 'A'),
            "4A" => Key::new(4, 'A'),
            "11A" => Key::new(11, 'A'),
            "6A" => Key::new(6, 'A'),
            "1A" => Key::new(1, 'A'),
            "8A" => Key::new(8, 'A'),
            "3A" => Key::new(3, 'A'),
            "10A" => Key::new(10, 'A'),
            _ => Err(CreateKeyError::InvalidKeyError),
        }
    }
}

impl Key {
    pub fn new(num: i8, letter: char) -> Result<Self, CreateKeyError> {
        if num > 12 || num <= 0 {
            return Err(CreateKeyError::InvalidNumberError);
        }

        if !['A', 'B'].contains(&letter) {
            return Err(CreateKeyError::InvalidLetterError);
        }

        Ok(Self { num, letter })
    }
    pub fn new_force(num: i8, letter: char) -> Self {
        Key::new(num, letter).unwrap()
    }
}

pub(crate) fn key_to_camelot(key: &str) -> Option<&str> {
    match key {
        "C Major" => Some("8B"),
        "C# Major" => Some("3B"),
        "D Major" => Some("10B"),
        "D# Major" => Some("5B"),
        "E Major" => Some("12B"),
        "F Major" => Some("7B"),
        "F# Major" => Some("2B"),
        "G Major" => Some("9B"),
        "G# Major" => Some("4B"),
        "A Major" => Some("11B"),
        "A# Major" => Some("6B"),
        "B Major" => Some("1B"),
        "C Minor" => Some("5A"),
        "C# Minor" => Some("12A"),
        "D Minor" => Some("7A"),
        "D# Minor" => Some("2A"),
        "E Minor" => Some("9A"),
        "F Minor" => Some("4A"),
        "F# Minor" => Some("11A"),
        "G Minor" => Some("6A"),
        "G# Minor" => Some("1A"),
        "A Minor" => Some("8A"),
        "A# Minor" => Some("3A"),
        "B Minor" => Some("10A"),
        _ => None,
    }
}
