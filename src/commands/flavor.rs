//		Imports
use twilight_model::{
	gateway::payload::incoming::MessageCreate,
	channel::message::{
		component::{
			Button, ButtonStyle, Component, ComponentType
		}
	}
};
use twilight_http::{
	request::channel::reaction::RequestReactionType
};

use crate::{
	BotResult,
	InteractionContext,
	GuildData
};	

//		Data
const emoji_yes: RequestReactionType = RequestReactionType::Unicode{};
const emoji_no: RequestReactionType = RequestReactionType::Unicode{};

//		Command
pub async fn flavor(
	ctx: InteractionContext, 
	msg: Box<MessageCreate>, 
	rest: &str
) -> BotResult<()> {
	//	Retrieve user data
	let guild_id = msg.guild_id.0;
	let user_id = msg.author.id.0;

	let guild_data = GuildData::read_file(guild_id).await?;
	let role_id = guild_data.flavor_map.get(&user_id).cloned();

	//	Get role (or create default)
	let role = match role_id {
		Some(id) => id,
		None => {	
			//	Message
			let make_msg = ctx.http.create_message(msg.channel_id)
				.content("It appears you do not have a flavor role. One has been created for you.")
				.await?;

			let new_role = ctx.http.create_role(guild_id)
				.color(0x8a8a8a)
				.name("flavorless")
				.await?;

			return Ok(())
		}
	};
	
	//	Split args
	let args = rest.split(' ');
	for mut arg in args {
		arg = arg.trim();
		let (subcommand, arg_rest) = match arg.split_once(':') {
			Some(res) => res,
			None => {
				return Err(());
			}
		};

		let result = match subcommand {
			"color" => {	
				//	Construct color
				arg_rest.len().checked_eq(6).ok_or_else(|| "Error: Invalid color")?;
				let color = u32::from_str_radix(arg_rest, 16).map_err(|| "Error: Invalid Color")?;

				ctx.http.update_role(guild_id, role_id)
        			.color(color)
        			.await?;

				Ok(())
			},
			"name" => {
				ctx.http.update_role(guild_id, role_id)
        			.name(arg_rest)
        			.await?;

				Ok(())
			},
			_ => Err("Invalid command".into())
		};
	}




	Ok(())
}