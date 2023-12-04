//		Imports
use std::{
	collections::HashMap,
	path::Path,
	fs, 
};
use serde::{Deserialize, Serialize};
use serde_json;

use twilight_model::id::{
	Id, marker::{
		GuildMarker,
		RoleMarker,
		UserMarker,
	}
};

use crate::{
	BotResult,
	InteractionContext
};

//		Data
#[derive(Debug, Serialize, Deserialize)]
pub struct GuildData {
	pub flavor_map: HashMap<Id<UserMarker>, Id<RoleMarker>>
}

//		Implementation
impl GuildData {
	pub fn new() -> Self {
		Self {
			flavor_map: HashMap::new()
		}
	}

	pub async fn read_file(
		guild_id: Id<GuildMarker>,
	) -> BotResult<Self> {
		//	Retrieve file
		let path = format!("data/guilds/guild_{:?}.json", guild_id);
		if Path::new(&path).exists() {
			let contents = fs::read_to_string(&path)
				.map_err(|e| format!("Error reading guild data: {}", e))?;
			let data: GuildData = serde_json::from_str(&contents)
				.map_err(|e| format!("Error parsing guild data JSON: {}", e))?;
	
			Ok(data)
		} else {
			Err(format!("Data for guild {:?} not found", path).into())
		}
	}
	
	pub async fn write_file(
		&self,
		guild_id: Id<GuildMarker>,
	) -> BotResult<()> {
		//	Construct path
		let path = format!("data/guilds/guild_{:?}.json", guild_id);
		if !Path::new(&path).exists() { 
	//		log::warn!(format!("Path {} does not exist", &path)) 
		}
	
		//	Write
		let serialized = serde_json::to_string(self)
			.map_err(|e| format!("Error serializing guild data: {}", e))?;
	
		fs::write(&path, serialized)
			.map_err(|e| format!("Error writing to file: {}", e))?;
	
		Ok(())
	}
}


