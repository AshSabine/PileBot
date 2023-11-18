//		Imports
use std::{};

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

//		Data


//		Functions
pub async fn handle_interaction(
	interaction: Interaction, 
	ctx: InteractionContext
) -> BotResult<()> {
	//	Handle different interaction types
	let inter = match interaction.kind {
		InteractionType::ApplicationCommandAutocomplete => {
			//	Return early, handle the autocomplete
			return handle_autocomplete(interaction, ctx).await
		}
		InteractionType::ApplicationCommand => interaction,
		InteractionType::Ping => {
			//	"Pong" back
			return Ok(())
		}
		_ => todo!(),
	};

	//	Retrieve data from interaction using refutable pattern match
	let Some(InteractionData::ApplicationCommand(data)) = inter.data.clone() else {
		return Err("No application data".into());
	};
	let name = data.name.as_str();

	//	Get handler
	let response = match name {
		"dice" => {
            todo!()
		}
		"role" => {
            todo!()
		}
		_ => {
			return Ok(());
		}
	};

	/*		Send response to HTTP client 
		Because this is at the lowest level of implementation w/i the Discord
		framework, you have to manually handle sending info back to the HTTP
		client. Other bots use a "context" struct for this.
		This creates an interface for using interactions (http.interaction),
		
	*/
	ctx.interaction().create_response(
		inter.id, 
		&inter.token, 
		&response
	).await?;

	Ok(())
}

pub async fn handle_autocomplete(
	ac: Interaction,
	ctx: InteractionContext
) -> BotResult<()> {

	Ok(())
}
