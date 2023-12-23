//		Imports
use std::{
	collections::HashMap,
	path::Path,
	fs, 
};
use serde::{Deserialize, Serialize};
use serde_json;

use time::OffsetDateTime;

use twilight_model::id::{
	Id, marker::{
		GuildMarker,
		RoleMarker,
		UserMarker,
	}
};

use crate::{
	BotResult
};

//		Guild Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildData {
	pub id: Id<GuildMarker>,

	pub flavor_map: HashMap<Id<UserMarker>, Id<RoleMarker>>,
}

impl GuildData {
	pub async fn new(id: Id<GuildMarker>) -> Self {
		let out = Self {
			id,

			flavor_map: HashMap::new(),
		};
		let _ = out.write_file().await;

		out
	}

	pub async fn read_file(
		guild_id: Id<GuildMarker>,
	) -> BotResult<Self> {
		//	Retrieve file
		let path = format!("data/guilds/guild_{}.json", guild_id.get());
		if Path::new(&path).exists() {
			let contents = fs::read_to_string(&path)
				.map_err(|e| format!("Error reading guild data: {}", e))?;
			let data: GuildData = serde_json::from_str(&contents)
				.map_err(|e| format!("Error parsing guild data JSON: {}", e))?;
	
			Ok(data)
		} else { Err(format!("Data at {:?} not found", path).into()) }
	}
	
	pub async fn write_file(
		&self,
	) -> BotResult<()> {
		//	Construct path
		let path = format!("data/guilds/guild_{}.json", self.id.get());
		if !Path::new(&path).exists() { 
	//		log::warn!(format!("Path {} does not exist", &path)) 
		}
	
		//	Write
		let serialized = serde_json::to_string(self)
			.map_err(|e| format!("Error serializing data: {}", e))?;
	
		fs::write(&path, serialized)
			.map_err(|e| format!("Error writing to file: {}", e))?;
	
		Ok(())
	}
}

//		Bot Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotData {
	pub issue_map: HashMap<i64, String>
}

impl BotData {
	pub async fn new() -> Self {
		let out = Self {
			issue_map: HashMap::new(),
		};
		
		out
	}

	pub async fn read_or_new() -> BotResult<Self> {
		//	Retrieve file
		let path = "data/bot_data.json".to_string();
		if Path::new(&path).exists() {
			let contents = fs::read_to_string(&path)
				.map_err(|e| format!("Error reading bot data: {}", e))?;
			let data: BotData = serde_json::from_str(&contents)
				.map_err(|e| format!("Error parsing bot data JSON: {}", e))?;
	
			Ok(data)
		} else {
			Ok(Self::new().await)
		}
	} 

	pub async fn write(
		&self,
	) -> BotResult<()> {
		//	Construct path
		let path = "data/bot_data.json".to_string();

		//	Write
		let serialized = serde_json::to_string_pretty(self)
			.map_err(|e| format!("Error serializing data: {}", e))?;
	
		fs::write(&path, serialized)
			.map_err(|e| format!("Error writing to file: {}", e))?;

		Ok(())
	}
}

