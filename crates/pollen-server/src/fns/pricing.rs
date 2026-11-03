//! The internal hosting cost endpoints (spec COST). Each answers only a request
//! carrying the configured pricing path in the `x-pricing-path` header, and
//! answers anything else as not found, so the public tool cannot reach a figure.

use axum::Json;
use axum::extract::{FromRequestParts, State};
use http::request::Parts;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::applications::load_ruleset;
use crate::db::Application;
use crate::error::{AppError, Result};
use crate::pricing::{
	CostBreakdown, CostInput, Display, Param, PlanCost, PlanOverrides, ServerBox, price_plan,
};
use crate::ruleset::Answers;
use crate::state::AppState;

/// The header carrying the pricing path segment from the internal edition's URL.
pub const PRICING_PATH_HEADER: &str = "x-pricing-path";

/// Proof that a request named the configured pricing path.
pub struct PricingAccess;

impl FromRequestParts<AppState> for PricingAccess {
	type Rejection = AppError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
		let given = parts
			.headers
			.get(PRICING_PATH_HEADER)
			.and_then(|v| v.to_str().ok())
			.unwrap_or_default();
		match &state.config.pricing_path {
			Some(expected) if constant_time_eq(given.as_bytes(), expected.as_bytes()) => {
				Ok(PricingAccess)
			}
			_ => Err(AppError::NotFound("not found".into())),
		}
	}
}

/// Compare without returning early on the first differing byte, so timing
/// does not reveal how much of a guess was right.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
	a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParamView {
	pub id: String,
	#[serde(flatten)]
	pub param: Param,
}

/// The model's configuration, for the calculator's controls and the
/// assumptions panel.
#[derive(Debug, Serialize, ToSchema)]
pub struct PricingModelView {
	pub model: String,
	pub bands: Vec<ServerBox>,
	pub facility: ServerBox,
	pub params: Vec<ParamView>,
	pub display: Display,
	pub max_cloud_facilities: u32,
	/// The configured share of the month a standard clone runs.
	pub clone_uptime: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ModelArgs {}

/// The pricing configuration.
#[utoipa::path(
    post, path = "/model", operation_id = "pricing_model", tag = "pricing",
    request_body = ModelArgs,
    responses((status = 200, body = PricingModelView), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn model(
	_access: PricingAccess,
	State(state): State<AppState>,
	Json(_args): Json<ModelArgs>,
) -> Result<Json<PricingModelView>> {
	let c = &state.pricing;
	Ok(Json(PricingModelView {
		model: c.model.clone(),
		bands: c.bands.clone(),
		facility: c.facility.clone(),
		params: c
			.params
			.entries()
			.into_iter()
			.map(|(id, p)| ParamView {
				id: id.to_string(),
				param: p.clone(),
			})
			.collect(),
		display: c.display.clone(),
		max_cloud_facilities: c.max_cloud_facilities,
		clone_uptime: c.params.clone_uptime.value,
	}))
}

/// Price inputs set directly (the calculator).
#[utoipa::path(
    post, path = "/cost", operation_id = "pricing_cost", tag = "pricing",
    request_body = CostInput,
    responses((status = 200, body = CostBreakdown), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn cost(
	_access: PricingAccess,
	State(state): State<AppState>,
	Json(input): Json<CostInput>,
) -> Result<Json<CostBreakdown>> {
	Ok(Json(state.pricing.cost(&input)?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlanArgs {
	pub id: Uuid,
	#[serde(default)]
	pub overrides: PlanOverrides,
}

/// Price a plan from its answers (the priced wizard).
#[utoipa::path(
    post, path = "/plan", operation_id = "pricing_plan", tag = "pricing",
    request_body = PlanArgs,
    responses((status = 200, body = PlanCost), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn plan(
	_access: PricingAccess,
	State(state): State<AppState>,
	Json(args): Json<PlanArgs>,
) -> Result<Json<PlanCost>> {
	let mut conn = state.db.get().await?;
	let app = Application::get(&mut conn, args.id).await?;
	let ruleset = load_ruleset(&mut conn, &app.config_hash).await?;
	let answers: Answers = serde_json::from_value(app.answers).map_err(AppError::custom)?;
	Ok(Json(price_plan(
		&state.pricing,
		&ruleset,
		&answers,
		&args.overrides,
	)?))
}

pub fn routes() -> OpenApiRouter<AppState> {
	OpenApiRouter::new()
		.routes(routes!(model))
		.routes(routes!(cost))
		.routes(routes!(plan))
}
