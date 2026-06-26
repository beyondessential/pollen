use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;
use crate::error::Result;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub db: Db,
}

impl AppState {
	pub async fn init() -> Result<Self> {
		let config = Config::from_env()?;
		let db = crate::db::init(&config.database_url);
		Ok(Self {
			config: Arc::new(config),
			db,
		})
	}
}
