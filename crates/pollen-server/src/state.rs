use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
}

impl AppState {
	pub async fn init() -> Result<Self> {
		Ok(Self {
			config: Arc::new(Config::from_env()?),
		})
	}
}
