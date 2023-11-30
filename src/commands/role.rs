//		Imports
use twilight_model::{
	application::interaction::{
		Interaction,
		InteractionType,
		InteractionData,
	}
};
use twilight_http::Client;

use crate::{
	BotResult, InteractionContext
};


//	Data
pub const NAME: &str = "role";

//	Implementation

