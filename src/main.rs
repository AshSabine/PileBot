//		Imports
use std::{
	env, 
	sync::Arc,
	error::Error, str::FromStr
};
use dotenv::dotenv;

use twilight_gateway::{Event, Shard, ShardId};
use twilight_model::{
	id::{
		Id,
		marker::ApplicationMarker
	},
	gateway::{Intents, payload::incoming::MessageCreate}, channel::Message
};
use twilight_http::{
	Client,
	client::InteractionClient
};

//  User stuff
mod data;

mod interaction;
use crate::{
	interaction::handle_interaction,
};

mod commands;

//		Data
pub type BotResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone)]
pub struct InteractionContext {
	http: Arc<Client>,
	app_id: Id<ApplicationMarker>
}

impl InteractionContext {
	fn new(
		http: Arc<Client>,
		app_id: Id<ApplicationMarker>, 
	) -> Self { InteractionContext{ http, app_id } }

	pub fn interaction(&self) -> InteractionClient<'_> {
		self.http.interaction(self.app_id)
	}
}


//		Functions
#[tokio::main]
async fn main() -> BotResult<()> {
	// Initialize the tracing subscriber.
	tracing_subscriber::fmt::init();

	//	Load env vars from the file
	dotenv().ok();

	//	Setup token / intents, login
	let token = env::var("DISCORD_TOKEN")
		.expect("Expected a token in the environment");
	println!("[MAIN] Token: {:?}", token);

	let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
	let http = Arc::new(Client::new(token.clone()));
	let app_id = http.current_user_application().await?.model().await?.id;

	let ctx = InteractionContext::new(http, app_id);

	//	Create shard
	let mut shard = Shard::new(ShardId::ONE, token, intents);

	//	Event loop
	let error = loop {
		let event = match shard.next_event().await {
			Ok(event) => event,
			Err(e) => {
				if e.is_fatal() { break e }
				continue;
			}
		};

		//	Spawn thread to handle event
		tokio::spawn(handle_event(event, ctx.clone()) );
	};

	Err(error.into())
}

pub async fn handle_event(
	event: Event,
	ctx: InteractionContext
) -> BotResult<()> {
	let result: BotResult<()> = handle_event_internal(event.clone(), ctx.clone()).await;
	let err_msg = match result {
		Err(e) => format!("{}", e),
		_ => { return Ok(()) }
	};

	match event {
		Event::MessageCreate(msg) => {
			ctx.http.create_message(msg.channel_id)
				.content(&err_msg)?.await?;
		}
		_ => {}
	}

	Ok(())
}

pub async fn handle_event_internal(
	event: Event, 
	ctx: InteractionContext
) -> BotResult<()> {
	match event {
		Event::Ready(_) => { println!("[EVNT] Shard is ready!") }

		//	This is a very rough message handling architecture which I intend to replace later.
		//	Note that the incoming content is boxed to optimize the memory footprint - the size
		//	of an enum is tied to the size of each of its members, boxing it keeps it consistent.
		Event::MessageCreate(msg) => {
			//*
			let lc = msg.content.clone().to_lowercase();
			
			//	Funny jokes
			if lc.contains("im") && (lc.split(' ').count() < 8) {
				//  Splits by im, takes the last element as a string & removes whitespace
				let text: &str = msg.content.split("im").last().unwrap().trim();

				//  Avoid sending a message if there isn't anything
				if text.is_empty() { return Ok(()) }

				let reply: String = format!("hi, {text}, i'm pilebot!");
				ctx.http.create_message(msg.channel_id).content(&reply)?.await?;
			}

			if lc.contains("pilebot why are you like this") {
				let reply: String = format!("i just am");
				ctx.http.create_message(msg.channel_id).content(&reply)?.await?;
			}
			
			//	Actual commands
			if let None = msg.content.split_once(' ') { return Ok(()) }
			let (mut name, mut rest) = msg.content.split_once(' ').unwrap();
			
			name = &name[1..];
			rest = rest.trim();

			match name {
				"dice" => commands::dice::dice(ctx, msg.clone(), rest).await?,
				"flavor" => commands::flavor::flavor(ctx, msg.clone(), rest).await?,
				"role" => {
					ctx.http.create_message(msg.channel_id)
						.content("role command unimplemented")?.await?;
				}
				
				_ => {}
			}
		}

		// "Interactions" are the proper term for Discord's slash commands. The ideal would be
		// to move to an interaction-based architecture rather than what currently exists.
		Event::InteractionCreate(interaction) => {
			match handle_interaction(interaction.0, ctx).await {
				Ok(_) => {}
				Err(e) => {}
			}
		}
 
		//	Handle other things.
		_ => {}
	}

	Ok(())
}
