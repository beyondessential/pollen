//! The internal data migration estimator's endpoints (spec MIG). Each answers
//! only a request carrying the configured migration path in the
//! `x-migration-path` header, and answers anything else as not found.

use axum::Json;
use axum::extract::{FromRequestParts, State};
use http::request::Parts;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::pricing::constant_time_eq;
use crate::data_migration::{
	AreaRow, CaseBands, Estimate, EstimateInput, ImportEstimate, ImportInput, Tier, TierRow,
};
use crate::error::{AppError, Result};
use crate::state::AppState;

/// The header carrying the migration path segment from the estimator's URL.
pub const MIGRATION_PATH_HEADER: &str = "x-migration-path";

/// Proof that a request named the configured migration path.
pub struct MigrationAccess;

impl FromRequestParts<AppState> for MigrationAccess {
	type Rejection = AppError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
		let given = parts
			.headers
			.get(MIGRATION_PATH_HEADER)
			.and_then(|v| v.to_str().ok())
			.unwrap_or_default();
		if constant_time_eq(
			given.as_bytes(),
			state.data_migration.access_path.as_bytes(),
		) {
			Ok(MigrationAccess)
		} else {
			Err(AppError::NotFound("not found".into()))
		}
	}
}

/// The model's configuration, for the estimator's controls.
#[derive(Debug, Serialize, ToSchema)]
pub struct MigrationModelView {
	pub model: String,
	pub version: String,
	pub tiers: Vec<TierRow>,
	/// The tier an estimate starts from when none is chosen.
	pub unsure_tier: Tier,
	pub areas: Vec<AreaRow>,
	/// The most a count answer is read as.
	pub max_count: u32,
	/// The most programs a client-prepared engagement is read as.
	pub max_projects: u32,
	/// The bands a program's rough case numbers are given in.
	pub case_bands: CaseBands,
	/// The bands of patients a day at the busiest site.
	pub patients_bands: CaseBands,
	/// The bands of years of visit history to migrate.
	pub years_bands: CaseBands,
	/// Staff to be migrated beyond which users take longer.
	pub many_users_over: u32,
	/// Separate installs from which a few, and from which many, add time.
	pub installs_some_from: u32,
	pub installs_many_from: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MigrationModelArgs {}

/// The migration configuration.
#[utoipa::path(
    post, path = "/model", operation_id = "data_migration_model", tag = "data-migration",
    request_body = MigrationModelArgs,
    responses((status = 200, body = MigrationModelView), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn model(
	_access: MigrationAccess,
	State(state): State<AppState>,
	Json(_args): Json<MigrationModelArgs>,
) -> Result<Json<MigrationModelView>> {
	let c = &state.data_migration;
	Ok(Json(MigrationModelView {
		model: c.model.clone(),
		version: c.version.clone(),
		tiers: c.tiers.clone(),
		unsure_tier: c.unsure_tier,
		areas: c.areas.clone(),
		max_count: c.max_count,
		max_projects: c.client_prepared.max_projects as u32,
		case_bands: c.client_prepared.case_bands,
		patients_bands: c.size.patients,
		years_bands: c.size.years,
		many_users_over: c.users.many_over,
		installs_some_from: c.source.installs.some_from,
		installs_many_from: c.source.installs.many_from,
	}))
}

/// Estimate a BES-led migration.
#[utoipa::path(
    post, path = "/estimate", operation_id = "data_migration_estimate", tag = "data-migration",
    request_body = EstimateInput,
    responses((status = 200, body = Estimate), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn estimate(
	_access: MigrationAccess,
	State(state): State<AppState>,
	Json(input): Json<EstimateInput>,
) -> Result<Json<Estimate>> {
	Ok(Json(state.data_migration.estimate(&input)))
}

/// Estimate a client-prepared import.
#[utoipa::path(
    post, path = "/import", operation_id = "data_migration_import", tag = "data-migration",
    request_body = ImportInput,
    responses((status = 200, body = ImportEstimate), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn import(
	_access: MigrationAccess,
	State(state): State<AppState>,
	Json(input): Json<ImportInput>,
) -> Result<Json<ImportEstimate>> {
	Ok(Json(state.data_migration.estimate_import(&input)))
}

pub fn routes() -> OpenApiRouter<AppState> {
	OpenApiRouter::new()
		.routes(routes!(model))
		.routes(routes!(estimate))
		.routes(routes!(import))
}
