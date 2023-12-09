//		Imports
use twilight_model::{
	guild::Role,
	gateway::payload::incoming::MessageCreate,
	channel::message::{
		Message,
		component::{
			Button, ButtonStyle, Component, ComponentType
		}
	},
	id::{
		Id, marker::{
			GuildMarker,
			RoleMarker,
			UserMarker
		}
	}
};
use twilight_http::{
	response::Response
};

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
	let guild_id: Id<GuildMarker> = msg.guild_id.expect("Message not in guild");
	let user_id: Id<UserMarker> = msg.author.id;

	//	Retrieve guild data
	let mut guild_data: GuildData = match GuildData::read_file(guild_id).await {
		Ok(res) => res,
		Err(_) => GuildData::new(guild_id).await
	};
	let role_id = guild_data.flavor_map.get(&user_id).cloned();

	//	Get role (or create default)
	let role_id: Id<RoleMarker> = match role_id {
		Some(id) => id,
		None => {	
			//	Message
			let _msg_response: Response<Message> = ctx.http.create_message(msg.channel_id)
				.content("it appears you do not have a flavor role; one has been created for you.")?
				.await?;

			//	Create role
			let role_response: Response<Role> = ctx.http.create_role(guild_id)
				.color(0x8a8a8a)
				.name("flavorless")
				.await?;

			//	Add role to user
			let new_role: Role = role_response.model().await?;
			ctx.http.add_guild_member_role(guild_id, user_id, new_role.id)
				.await?;

			//	Put role in guild data map
			guild_data.flavor_map.insert(user_id, new_role.id);
			guild_data.write_file().await?;

			//push_role_forward(ctx.clone(), new_role.id, guild_data).await?;

			return Ok(())
		}
	};
	
	//	Split args
	let args = rest.split(' ');
	for mut arg in args {
		arg = arg.trim();
		match arg.split_once(':') {
			Some((sub, rem)) => match sub {
				"color" => if rem.len() == 6 {
					let color = u32::from_str_radix(rem, 16).map_err(|_| "Invalid color")?;
	
					ctx.http.update_role(guild_id, role_id)
						.color(Some(color))
						.await?;
				} else { return Err("Invalid color".into()) },
				"name" => {
					ctx.http.update_role(guild_id, role_id)
						.name(Some(rem))
						.await?;
				},
				_ => return Err("Invalid command".into())
			},
			None => {
				

				let flavor_msg = match arg {
					"color" => format!("color: {}", 0x000000),
					"name" => format!("name: {}", ""),
					_ => return Err("Invalid command".into())
				};

				ctx.http.create_message(msg.channel_id)
					.content(&flavor_msg)?
					.await?;
			}
		};
	}

	//push_role_forward(ctx.clone(), role_id, guild_data).await?;

	Ok(())
}

async fn push_role_forward(
	ctx: InteractionContext, 
	role_id: Id<RoleMarker>,
	guild_data: GuildData,
) -> BotResult<()> {
	//	Get guild ID from data
	let guild_id: Id<GuildMarker> = guild_data.id;

	//	Get roles in the guild
	let guild_roles: Vec<Role> = ctx.http.roles(guild_id).await?.model().await?;
	let flavor_roles: Vec<&Role> = guild_roles.iter()
		.filter(|&r| 
			guild_data.flavor_map.values()
			.any(|&f| f == r.id)
		).collect();

	//	Find flavor positions (determining how the role order will be generated)
	let role_pos: u64 = flavor_roles.iter()
    	.find_map(|&r| if r.id == role_id { Some(r.position as u64) } else { None })
    	.ok_or("Could not find role")?;

	let min_pos: u64 = flavor_roles.iter()
		.map(|r| r.position as u64)
		.min()
		.unwrap_or(0);

	//	Create role order vector
	let mut role_order: Vec<(Id<RoleMarker>, u64)> = guild_roles.iter()
    	.map(|r| (r.id, r.position as u64))
		.collect();
	role_order.sort_by_key(|&(_, position)| position);

	let (role_id, _) = role_order.remove(role_pos as usize);
	role_order.insert(min_pos as usize, (role_id, min_pos));

	for (_, pos) in &mut role_order {
		if *pos >= min_pos {
			*pos += 1;
		}
	}

	//	Update role order
	ctx.http.update_role_positions(guild_id, &role_order).await?;

	Ok(())
}