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
	CostBreakdown, CostInput, EstimatorView, PlanCost, PlanOverrides, ServerBox, SizeArgs,
	SizeEstimate, price_plan,
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

/// The model's configuration, for the calculator's controls.
#[derive(Debug, Serialize, ToSchema)]
pub struct PricingModelView {
	pub model: String,
	pub bands: Vec<ServerBox>,
	pub max_cloud_facilities: u32,
	/// The sizing questions the calculator's estimator asks.
	pub estimator: EstimatorView,
	/// For each band, in order, the facility servers if they all run in BES
	/// cloud.
	pub facility_guess_by_band: Vec<u32>,
	/// The share of the facility servers in BES cloud for a mix of both.
	pub mix_share: f64,
	/// The wizard question the cloud facility slider takes the place of.
	pub facility_slider_question: String,
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
	let ruleset = state.default_ruleset.load_full();
	let ruleset = &ruleset.ruleset;
	Ok(Json(PricingModelView {
		model: c.model.clone(),
		bands: c.bands.clone(),
		max_cloud_facilities: c.max_cloud_facilities,
		estimator: c.estimator_view(ruleset),
		facility_guess_by_band: c.band_facility_guesses(ruleset),
		mix_share: c.wizard.cloud_share.mix_share,
		facility_slider_question: c.wizard.slider_in_place_of.clone(),
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

/// Work out the size band, and facility servers, from sizing answers (the
/// calculator's estimator).
#[utoipa::path(
    post, path = "/size", operation_id = "pricing_size", tag = "pricing",
    request_body = SizeArgs,
    responses((status = 200, body = SizeEstimate), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn size(
	_access: PricingAccess,
	State(state): State<AppState>,
	Json(args): Json<SizeArgs>,
) -> Result<Json<SizeEstimate>> {
	let ruleset = state.default_ruleset.load_full();
	Ok(Json(state.pricing.estimate_size(&ruleset.ruleset, &args)))
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
		.routes(routes!(size))
		.routes(routes!(plan))
}
