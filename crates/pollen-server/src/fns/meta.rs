use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::Result;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VersionInfo {
	pub name: String,
	pub version: String,
}

/// Service name and version. Doubles as a trivial reachability check.
#[utoipa::path(
    post,
    path = "/version",
    operation_id = "meta_version",
    tag = "meta",
    responses((status = 200, body = VersionInfo)),
)]
pub async fn version(State(_state): State<AppState>) -> Result<Json<VersionInfo>> {
	Ok(Json(VersionInfo {
		name: env!("CARGO_PKG_NAME").to_owned(),
		version: env!("CARGO_PKG_VERSION").to_owned(),
	}))
}

pub fn routes() -> OpenApiRouter<AppState> {
	OpenApiRouter::new().routes(routes!(version))
}
