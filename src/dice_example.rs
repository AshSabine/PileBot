use std::num::ParseIntError;
use std::str::FromStr;
use thiserror; // 1.0.50
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roll {
    pub num_of_dice: u32,
    pub num_of_sides: u32,
    pub operation: Option<Operation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Advantage,
    Extra(i32),
    Reroll(u32),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseRollError {
    #[error("Missing character")]
    MissingChar,
    #[error("Unrecognized operation")]
    UnrecognizedOp,
    #[error(transparent)]
    ParseError(#[from] ParseIntError),
}

impl FromStr for Operation {
    type Err = ParseRollError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args_text = s[1..].trim();
        match &s[0..1] {
            "r" => Ok(Operation::Reroll(args_text.parse()?)),
            "x" => {
                let end = args_text.find('+').unwrap_or(s.len());
                Ok(Operation::Extra(args_text[..end].trim().parse()?))
            },
            "a" => Ok(Operation::Advantage),
            _ => Err(ParseRollError::UnrecognizedOp),
        }
    }
}

impl FromStr for Roll {
    type Err = ParseRollError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num_of_dice_str, rest_of_s) = s.split_once('d').ok_or(ParseRollError::MissingChar)?;
        
        let (num_of_sides_str, operation) = match rest_of_s.find(['r', 'x', 'a']) {
            None => (rest_of_s, None),
            Some(idx_of_opname) => {
                let (left_str, right_str) = rest_of_s.split_at(idx_of_opname);
                (left_str, Some(Operation::from_str(right_str)?))
            }
        };

        Ok(Roll {
            num_of_dice: num_of_dice_str.trim().parse()?,
            num_of_sides: num_of_sides_str.trim().parse()?,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_roll() {
        assert_eq!(
            "80d200".parse(),
            Ok(Roll {
                num_of_dice: 80,
                num_of_sides: 200,
                operation: None,
            })
        );
        assert_eq!(
            "3d6r1".parse(),
            Ok(Roll {
                num_of_dice: 3,
                num_of_sides: 6,
                operation: Some(Operation::Reroll(1)),
            })
        );
        assert_eq!(
            "2d4x2+".parse(),
            Ok(Roll {
                num_of_dice: 2,
                num_of_sides: 4,
                operation: Some(Operation::Extra(2)),
            })
        );
        assert_eq!(
            "1d20a".parse(),
            Ok(Roll {
                num_of_dice: 1,
                num_of_sides: 20,
                operation: Some(Operation::Advantage),
            })
        );
    }

    #[test]
    fn test_parse_roll_with_whitespace() {
        assert_eq!(
            "0 d 10000".parse(),
            Ok(Roll {
                num_of_dice: 0,
                num_of_sides: 10000,
                operation: None,
            })
        );
        assert_eq!(
            "3d6 r 1".parse(),
            Ok(Roll {
                num_of_dice: 3,
                num_of_sides: 6,
                operation: Some(Operation::Reroll(1)),
            })
        );
        assert_eq!(
            "2d        4 x 2+".parse(),
            Ok(Roll {
                num_of_dice: 2,
                num_of_sides: 4,
                operation: Some(Operation::Extra(2)),
            })
        );
        assert_eq!(
            "1d   20  a".parse(),
            Ok(Roll {
                num_of_dice: 1,
                num_of_sides: 20,
                operation: Some(Operation::Advantage),
            })
        );
    }
}

// [count]d[size]<operation>[args](direction)
// default size is a d1 so roll 3d6 + 4 parses correctly
// operations like rerolling numbers (r) adding extra die (e or x) advantage/disadvantage (a & d)
