//		Imports
use std::{
	fmt, cmp::{min, max},
	str::FromStr,
	num::ParseIntError,
};
use rand::Rng;

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
	BotResult,
	InteractionContext
};

//		Command
pub async fn dice(
	ctx: InteractionContext, 
	msg: Box<MessageCreate>, 
	rest: &str
) -> BotResult<()> {
	//  Parse roll
	match DiceCommand::from_str(rest) {
		Ok(to_roll) => {
			let roll: i32 = to_roll.roll();
			let reply: String = format!("you rolled: {roll}");
			ctx.http.create_message(msg.channel_id).content(&reply)?.await?;

			return Ok(())
		}
		Err(e) => {
			ctx.http.create_message(msg.channel_id).content(format!("{}", e).into())?.await?;

			return Ok(())
		}
	}
}

//		Implementation
//  Structs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceCommand {
	dice:Vec<Dice>
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Dice {
	op:ArithOp,

	count:i32,
	sides:i32,

	args:Vec<DiceArg>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiceArg {
	Advantage(bool),
	Extra(i32, i32),
	Reroll(i32, i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArithOp {
	Add,
	Sub,
	None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseRollError {
	MissingChar,
	UnrecognizedOp(String),
	ParseIntError(ParseIntError),
}

impl fmt::Display for ParseRollError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::MissingChar => write!(f, "Expected char in string"),
			Self::UnrecognizedOp(c) => write!(f, "Unrecognised operation \"{}\"", c),
			Self::ParseIntError(e) => write!(f, "Error parsing input: {}", e),
		}
	}
}

impl From<ParseIntError> for ParseRollError {
    fn from(value: ParseIntError) -> Self {
        ParseRollError::ParseIntError(value)
    }
}

//  Functions
impl FromStr for DiceCommand {
	type Err = ParseRollError;

	fn from_str(w: &str) -> Result<Self, Self::Err> {
		let mut s = w.to_owned();
		s.retain(|c| !c.is_whitespace());

		//  Overall command vars
		let mut dice: Vec<Dice> = vec![]; 

		for die in s.split_inclusive(['+', '-']) {
			println!("die: {}", die);
			let next_die = Dice::from_str(die)?;
			dice.push(next_die);
		}
		Ok(DiceCommand{ dice })
	}
}

//*
impl FromStr for Dice {
	type Err = ParseRollError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		//	Split at 'd' or return d1s
		let (str_count, mut str_rest) = match s.split_once('d') {
			Some(res) => res,
			None => {
				let next_op = ArithOp::from_str(s).map_err(|_| ParseRollError::MissingChar)?;
				let bias = s.strip_suffix(['+', '-']).unwrap_or(s).parse::<i32>()?;
				
				return Ok(Dice{
					op: next_op,
					count: bias,
					sides: 1,
					args: vec![]
				})
			}
		};

		println!("[from_str] str_count: {:?}, str_rest: {:?}", str_count, str_rest);

		//	Slice string to remove operation
		let next_op = ArithOp::from_str(str_rest)?;
		str_rest = match next_op {
			ArithOp::None => str_rest,
			_ => &str_rest[..str_rest.len()-1]
		};		

		println!("[from_str] str_rest (post-cull): {:?}", str_rest);

		//	Seperate args & size string
		let mut last: usize = 0;
		let mut result: Vec<&str> = Vec::new();
		for (index, sep) in str_rest.match_indices(|c| c=='r'||c=='x'||c=='a'||c=='d') {
			match last {
				0 => { result.push(&str_rest[..index]); }
				l => { result.push(&str_rest[l-1..index]); }
			}
			last = index + sep.len();
		} 
		result.push(&str_rest[last..]);
		println!("[from_str] result: {:?}", result);

		let str_sides = result[0];
		println!("[from_str] str_sides: {:?}", str_sides);

		//	Iterator magic DiceArgs parse 
		let mut args: Vec<DiceArg> = result[1..]
			.into_iter()
			.map(|arg| DiceArg::from_str(arg))
			.collect::<Result<_, _>>()?;
		println!("[from_str] args: {:?}", args);

		args = merge_args(args);
		println!("[from_str] args (post-merge): {:?}", args);

		//	Parse numbers
		let count = str_count.parse::<i32>()?;
		let sides = str_sides.parse::<i32>()?;

		//	Return
		Ok(Dice{ 
			op: next_op, 
			count, 
			sides, 
			args 
		})
	}
}
// */

//	Moving extract_args to here.
impl FromStr for DiceArg {
	type Err = ParseRollError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let arg_char: &str = &s[0..1];
		let arg_rest: &str = &s[1..];
		match arg_char {
			//	Advantage
			"a"|"d" => Ok( DiceArg::Advantage(arg_char == "a") ),

			//	Extra/Reroll
			"x"|"r" => {
				let parts: Vec<&str> = arg_rest.split('.').collect();
			
				let left: i32 = parts[0].parse::<i32>().unwrap_or(0);
				let right: i32 = parts.last()
					.map(|part| part.parse::<i32>().unwrap_or(100))
					.unwrap_or(100);

				match arg_char {
					"x" => Ok(DiceArg::Extra(left, right)),
					"r" => Ok(DiceArg::Reroll(left, right)),
					other => Err(ParseRollError::UnrecognizedOp(other.to_owned()))
				}
			}

			//	Other
			other => Err(ParseRollError::UnrecognizedOp(other.to_owned()))
		}
	}

}

impl FromStr for ArithOp {
	type Err = ParseRollError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some(i) = s.rfind(|c| c == '+' || c == '-') {
			return Ok(match &s[i..] {
				"+" => ArithOp::Add,
				"-" => ArithOp::Sub,
				_ => ArithOp::None
			})
		} 
		
		Ok(ArithOp::None)
	}
}

fn merge_args(args: Vec<DiceArg>) -> Vec<DiceArg> {
	let mut out: Vec<DiceArg> = vec![];

	let mut extra: (i32, i32) = (0, 100);
	let mut reroll: (i32, i32) = (0, 100);

	let mut adv: i32 = 0;
	for arg in args {
		match arg {
			DiceArg::Advantage(polarity) => { adv += if polarity {1} else {-1}; },
			DiceArg::Extra(left, right) => {
				if left == right { out.push(arg); continue; }
				extra.0 = extra.0.max(left);
				extra.1 = extra.1.min(right);
			},
			DiceArg::Reroll(left, right) => {
				if left == right { out.push(arg); continue; }
				reroll.0 = reroll.0.max(left);
				reroll.1 = reroll.1.min(right);
			}
		}
	}

	if adv != 0 { out.push(DiceArg::Advantage(adv > 0)); }
	if extra != (0, 100) { out.push(DiceArg::Extra(extra.0, extra.1)); }
	if reroll != (0, 100) { out.push(DiceArg::Reroll(reroll.0, reroll.1)); }

	out
}

impl DiceCommand {
	pub fn roll(&self) -> i32 {
		let mut rng = rand::thread_rng();
		let mut sum: i32 = 0;
		let mut next_op = ArithOp::Add;

		for die in &self.dice {
			let mut count: i32 = die.count;
			
			if die.sides == 1 {
				sum += match next_op{ ArithOp::Add => count, ArithOp::Sub => -count, _ => 0 };
				next_op = die.op;
				continue
			}

			while count > 0 {
				let mut roll: i32 = rng.gen_range(1..=die.sides);

				if let Some(DiceArg::Advantage(p)) = die.args.iter()
				.find(|&a| matches!(a, DiceArg::Advantage(_))) {
					let roll2: i32 = rng.gen_range(1..=die.sides);
					roll = match p {
						true => max(roll, roll2),
						false => min(roll, roll2)
					};
				}
				
				if die.args.iter().any(|a| 
					matches!(a, DiceArg::Reroll(l, r) 
					if (*l..=*r).contains(&roll))) 
				{ 
					roll = rng.gen_range(1..=die.sides); 
					if let Some(DiceArg::Advantage(p)) = die.args.iter()
					.find(|&a| matches!(a, DiceArg::Advantage(_))) {
						let roll2: i32 = rng.gen_range(1..=die.sides);
						roll = match p {
							true => max(roll, roll2),
							false => min(roll, roll2)
						};
					}
				}

				if die.args.iter().any(|a| 
					matches!(a, DiceArg::Extra(l, r) 
					if (*l..=*r).contains(&roll))) 
				{ count += 1; }

				sum += match next_op{ ArithOp::Add => roll, ArithOp::Sub => -roll, _ => 0 };
				count -= 1;
			}

			next_op = die.op;
		}

		sum
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn single() {
        let input = "2d6";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 1);
        assert_eq!(command.dice[0].count, 2);
        assert_eq!(command.dice[0].sides, 6);
    }

	
    #[test]
    fn multi() {
        let input = "2d6+1d8";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 2);
        assert_eq!(command.dice[0].count, 2);
        assert_eq!(command.dice[0].sides, 6);
        assert_eq!(command.dice[1].count, 1);
        assert_eq!(command.dice[1].sides, 8);
    }

	#[test]
	fn add() {
		let input = "2d6+1";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 2);
        assert_eq!(command.dice[0].count, 2);
        assert_eq!(command.dice[0].sides, 6);
		assert_eq!(command.dice[1].count, 1);
        assert_eq!(command.dice[1].sides, 1);
	}

	/*
    #[test]
    fn advantage() {
        let input = "2d6a";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 1);
        assert_eq!(command.dice[0].args, vec![DiceArg::Advantage(true)]);
    }

    #[test]
    fn reroll() {
        let input = "2d6r..2";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 1);
        assert_eq!(command.dice[0].args, vec![DiceArg::Reroll(0, 2)]);
    }

    #[test]
    fn extra() {
        let input = "2d6x4..";
        let result = DiceCommand::from_str(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.dice.len(), 1);
        assert_eq!(command.dice[0].args, vec![DiceArg::Extra(0, 4)]);
    }

    #[test]
    fn invalid() {
        let input = "invalid";
        let result = DiceCommand::from_str(input);
        assert!(result.is_err());

        match result.err().unwrap() {
            ParseRollError::UnrecognizedOp(op) => assert_eq!(op, "invalid"),
            _ => panic!("Expected UnrecognizedOp error"),
        }
    }
	// */
}