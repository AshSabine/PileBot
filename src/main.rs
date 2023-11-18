//		Imports
use std::{
	env, 
	sync::Arc,
	error::Error, str::FromStr
};

use twilight_gateway::{Event, Shard, ShardId};
use twilight_model::{
	id::{
		Id,
		marker::ApplicationMarker
	},
	gateway::{Intents}
};
use twilight_http::{
	Client,
	client::InteractionClient
};

//  User stuff
mod interaction;

//		Data
pub type BotResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

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
	match event {
		Event::Ready(_) => { println!("[EVNT] Shard is ready!") }

		//	This is a very rough message handling architecture which I intend to replace later.
		//	Note that the incoming content is boxed - I don't know why, should figure it out.
		Event::MessageCreate(msg) => {
			
			let lc = msg.content.clone().to_lowercase();
			if lc.contains("im") && (lc.split(' ').count() < 4) {
				//  Splits by im, takes the last element as a string & removes whitespace
				let text: &str = msg.content.split("im").last().unwrap().trim();

				//  Avoid sending a message if there isn't anything
				if text.is_empty() { return Ok(()) }

				let reply: String = format!("hi, {text}, i'm pilebot!");
				ctx.http.create_message(msg.channel_id).content(&reply)?.await?;
			}
			
			if lc.starts_with("!dice ") {
				//  Splits by command, takes the last element as a string & removes whitespace
				let roll_str: &str = msg.content.split("!roll ").last().unwrap().trim();

				//  Avoid sending a message if there isn't anything
				if roll_str.is_empty() { return Ok(()) }

				//  Parse roll
				if let Ok(to_roll) = DiceCommand::from_str(roll_str) {
					let roll: i32 = to_roll.roll();
					let reply: String = format!("you rolled: {roll}");
					ctx.http.create_message(msg.channel_id).content(&reply)?.await?;

					return Ok(())
				}

				ctx.http.create_message(msg.channel_id).content("error parsing roll")?.await?;
			}			
		}

		// "Interactions" are the proper term for Discord's slash commands. The ideal would be
		// to move to an interaction-based architecture rather than what currently exists.
		Event::InteractionCreate(interaction) => {
			match handle_interaction(interaction.0, ctx) {
				Ok(_) => {}
				Err(e) => {}
			}
		}
 
		//	Handle other things.
		_ => {}
	}

	Ok(())
}

/*	Coming later b/c currently this causes an error & I don't understand the Tokio framework very well.
//	Sub functions
async fn handle_msg(
	msg: &MessageCreate,
	http: &Arc<Client>,
) -> Result<(), Box<dyn Error + Send + Sync>> {

}
 */