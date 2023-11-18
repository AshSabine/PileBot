//		Imports
use twilight_model::{
	application::interaction::{
		Interaction,
		InteractionType,
		InteractionData, 
		application_command::CommandDataOption,
	},
	http::interaction::InteractionResponse
};

use crate::{
	BotResult
};

//		Data
pub const NAME: &str = "dice";

//		Implementation
fn dice(options: &[CommandDataOption]) -> InteractionResponse {
	let parse = options.iter().find(|o| o.name == "parse");


	Ok(())
}