//  Imports
use std::{
	fmt,
	error::Error,
	str::FromStr,
	num::ParseIntError,
};
use rand::Rng;

//  Structs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceCommand {
	dice:Vec<Dice>,
	stored_sum:i32
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
enum ParseRollError {
	MissingChar(String, usize),
	UnrecognizedOp(String),
}
impl fmt::Display for ParseRollError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::MissingChar(s, i) => write!(f, "Expected char here: {} <- ", &s[..*i]),
			Self::UnrecognizedOp(c) => write!(f, "Unrecognised operation \"{}\"", c),
		}
	}
}


//  Functions
impl FromStr for DiceCommand {
	type Err = ParseIntError;

	fn from_str(w: &str) -> Result<Self, Self::Err> {
		let mut s = w.to_owned();
		s.retain(|c| !c.is_whitespace());

		//  Overall command vars
		let mut dice: Vec<Dice> = vec![]; 
		let mut stored_op: ArithOp = ArithOp::Add;
		let mut stored_sum: i32 = 0;

		for die in s.split_inclusive(['+', '-']) {
			match die.split_once('d') {
				Some((lhs, rhs)) => {
					//	Get next operation
					let (next_op,slice) = extract_op(rhs);
					
					//	Read args
					let (args, sli) = extract_args(slice);

					


					//	Parse 
					let count = lhs.parse::<i32>().expect("Failed to parse dice left input.");
					let sides = sli.parse::<i32>().expect("Failed to parse dice right input.");

					//	Operate
					dice.push( Dice{ op:stored_op, count, sides, args:merge_args(args) } );
					stored_op = next_op;
				},
				None => {
					//	Get next operation
					let (next_op, slice) = extract_op(die); 

					//	Parse
					let count = slice.parse::<i32>().expect("Failed to parse sum input.");

					//	Operate
					match stored_op {
						ArithOp::Add => { stored_sum += count },
						ArithOp::Sub => { stored_sum -= count },
						ArithOp::None => {}
					};

					stored_op = next_op;
				}
			}
		}

		Ok(DiceCommand{ dice, stored_sum })
	}

	
}

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

fn extract_op(s: &str) -> (ArithOp, &str) {
	if let Some(i) = s.rfind(|c| c == '+' || c == '-') {
		let out = match &s[i..] {
			"+" => ArithOp::Add,
			"-" => ArithOp::Sub,
			_ => ArithOp::None
		};

		(out, &s[..i])
	} else {
		(ArithOp::None, &s)
	}
}

fn extract_args(s: &str) -> (Vec<DiceArg>, &str) {
	//	Seperate to args
	let mut last: usize = 0;
	let mut result: Vec<&str> = Vec::new();
	for (index, sep) in s.match_indices(|c| c=='r'||c=='x'||c=='a'||c=='d') {
		match last {
			0 => { result.push(&s[..index]); }
			l => { result.push(&s[l-1..index]); }
		}
		last = index + sep.len();
	} 
	result.push(&s[last..]);
	let slice = result[0];
	
	//	Parse args
	let mut args: Vec<DiceArg> = vec![];
	for arg in result {
		let next_char: Option<char> = arg.chars().next();

		if let Some('a')|Some('d') = next_char {
			args.push(DiceArg::Advantage(next_char == Some('a')));
		}

		if let Some('x')|Some('r') = next_char {
			let parts: Vec<&str> = arg[1..].split('.').collect();
			
			let left: i32 = parts[0].parse::<i32>().unwrap_or(0);
			let right: i32 = parts.last()
				.map(|part| part.parse::<i32>().unwrap_or(100))
				.unwrap_or(100);
			
			args.push(match next_char {
				Some('x') => DiceArg::Extra(left, right),
				Some('r') => DiceArg::Reroll(left, right),
				_ => panic!("Unexpected error while parsing dice.")
			});
		};
	}

	(args, slice)
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

	/*
	out.retain(|arg| match arg {
		DiceArg::Extra(l, _) => !(extra.0 ..= extra.1).contains(l),
		DiceArg::Reroll(l, _) => !(reroll.0 ..= reroll.1).contains(l),
		_ => false
	});  
	*/

	if adv != 0 { out.push(DiceArg::Advantage(adv > 0)); }
	if extra != (0, 100) { out.push(DiceArg::Extra(extra.0, extra.1)); }
	if reroll != (0, 100) { out.push(DiceArg::Reroll(reroll.0, reroll.1)); }

	out
}

impl DiceCommand {
	pub fn roll(&self) -> i32 {
		let mut rng = rand::thread_rng();
		let mut sum: i32 = self.stored_sum;

		for die in &self.dice {
			let mut count: i32 = die.count;

			while count > 0 {
				let mut roll: i32 = rng.gen_range(1..=die.sides);

				if die.args.iter().any(|a| 
					matches!(a, DiceArg::Reroll(l, r) 
					if (*l..=*r).contains(&roll))) 
				{ roll = rng.gen_range(1..=die.sides); }

				if die.args.iter().any(|a| 
					matches!(a, DiceArg::Extra(l, r) 
					if (*l..=*r).contains(&roll))) 
				{ count += 1; }

				sum += match die.op{ ArithOp::Add => roll, ArithOp::Sub => -roll, _ => 0 };
				count -= 1;
			}
		}

		sum
	}
}