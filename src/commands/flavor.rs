//		Imports
use twilight_model::{
	gateway::payload::incoming::MessageCreate,
	channel::message::
		component::{
			Button, ButtonStyle, Component, ComponentType
		}
};
//use twilight_http::{};

use crate::{
	BotResult,
	InteractionContext,
	data::GuildData
};	

//		Data
//const emoji_yes: RequestReactionType = RequestReactionType::Unicode{};
//const emoji_no: RequestReactionType = RequestReactionType::Unicode{};

//		Command
pub async fn flavor(
	ctx: InteractionContext, 
	msg: Box<MessageCreate>, 
	rest: &str
) -> BotResult<()> {
	//	Retrieve user data
	let guild_id = msg.guild_id.expect("Message not in guild");
	let user_id = msg.author.id;

	//	Retrieve guild data
	let guild_data = match GuildData::read_file(guild_id).await {
		Ok(res) => res,
		Err(_) => GuildData::new(guild_id).await
	};
	let role_id = guild_data.flavor_map.get(&user_id).cloned();

	//	Get role (or create default)
	let role_id = match role_id {
		Some(id) => id,
		None => {	
			//	Message
			let make_msg = ctx.http.create_message(msg.channel_id)
				.content("It appears you do not have a flavor role. One has been created for you.")?
				.await?;

			let new_role = ctx.http.create_role(guild_id)
				.color(0x8a8a8a)
				.name("flavorless")
				.await?;

			//guild_data.flavor_map.insert(user_id, new_role.id);

			return Ok(())
		}
	};
	
	//	Split args
	let args = rest.split(' ');
	for mut arg in args {
		arg = arg.trim();
		let (subcommand, arg_rest) = match arg.split_once(':') {
			Some(res) => res,
			None => { return Err("Splitting error".into()) }
		};

		let _ = match subcommand {
			"color" => if arg_rest.len() == 6 {
				let color = u32::from_str_radix(arg_rest, 16).map_err(|_| "Invalid color")?;

				ctx.http.update_role(guild_id, role_id)
					.color(Some(color))
					.await?;

				Ok(())
			} else { Err("Invalid color") },
			"name" => {
				ctx.http.update_role(guild_id, role_id)
        			.name(Some(arg_rest))
        			.await?;

				Ok(())
			},
			_ => Err("Invalid command".into())
		}?;
	}

	Ok(())
}