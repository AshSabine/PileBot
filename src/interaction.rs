//		Imports
use std::{};

use twilight_model::{
	application::interaction::{
		Interaction,
		InteractionType,
		InteractionData,
	}, 
	http::interaction::InteractionResponse
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
		InteractionType::MessageComponent => todo!(),
		InteractionType::Ping => {
			//	"Pong" back
			return Ok(())
		}
		_ => todo!(),
	};

	let response: InteractionResponse = match inter.data.clone().ok_or("No application data")? {
		InteractionData::ApplicationCommand(data) => {
			let name = data.name.as_str();

			//	Get handler
			match name {
				"dice" => {
					todo!()
				}
				"role" => {
					todo!()
				}
				_ => {
					todo!()
				}
			}
		},
		InteractionData::MessageComponent(data) => {
			todo!()
		},
		_ => { 
			return Err("No application data".into())
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