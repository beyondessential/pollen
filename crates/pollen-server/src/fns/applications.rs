//! Application lifecycle: create a draft (optionally previewing a ruleset
//! branch), read it with its evaluation, edit a draft's answers, finalise it,
//! and fork a new version (spec WIZ, Artifact lifecycle).

use axum::Json;
use axum::extract::State;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::db::{Application, ApplicationStatus, ConfigRow};
use crate::error::{AppError, Result};
use crate::ruleset::{
	Answers, Evaluation, Opt, Question, QuestionKind, ResolvedRuleset, Ruleset, evaluate, migrate,
};
use crate::state::AppState;

/// The full state of an application: its lifecycle fields, the questions to
/// render, the current answers, the evaluation (consequences, verdict, derived
/// values, visible questions, guidance), and — on a fork — what changed.
#[derive(Debug, Serialize, ToSchema)]
pub struct AppView {
	pub id: Uuid,
	pub status: ApplicationStatus,
	pub parent_id: Option<Uuid>,
	pub created_at: Timestamp,
	pub finalised_at: Option<Timestamp>,
	pub config_hash: String,
	/// True for a draft bound to a ruleset other than the current bundled
	/// default — i.e. a newer default is available to update to.
	pub update_available: bool,
	pub questions: Vec<QuestionView>,
	pub answers: Value,
	pub evaluation: Evaluation,
	pub migration: Option<MigrationView>,
}

/// A question's render metadata (the engine decides visibility; see
/// `Evaluation::visible_questions`).
#[derive(Debug, Serialize, ToSchema)]
pub struct QuestionView {
	pub id: String,
	pub kind: QuestionKind,
	pub label: String,
	pub help: Option<String>,
	pub options: Vec<Opt>,
}

impl From<&Question> for QuestionView {
	fn from(q: &Question) -> Self {
		Self {
			id: q.id.clone(),
			kind: q.kind,
			label: q.label.clone(),
			help: q.help.clone(),
			options: q.options.clone(),
		}
	}
}

/// What a fork's migration changed (spec WIZ, stable-id migration).
#[derive(Debug, Serialize, ToSchema)]
pub struct MigrationView {
	pub dropped: Vec<String>,
	pub new_questions: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArgs {
	/// A ruleset branch to preview; omitted binds the bundled default.
	#[serde(default)]
	pub config_branch: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetArgs {
	pub id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchArgs {
	pub id: Uuid,
	/// Answers keyed by question id; each value is an option id or a list of them.
	pub answers: Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FinaliseArgs {
	pub id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ForkArgs {
	pub id: Uuid,
	/// A ruleset branch to rebind to; omitted keeps the parent's ruleset.
	#[serde(default)]
	pub config_branch: Option<String>,
	/// Rebind to the current bundled default ruleset (takes precedence over
	/// `config_branch`). Used to update a stale draft to the latest default.
	#[serde(default)]
	pub to_default: bool,
}

/// Create a new draft, binding the bundled default ruleset or a previewed branch.
#[utoipa::path(
    post, path = "/create", operation_id = "applications_create", tag = "applications",
    request_body = CreateArgs,
    responses((status = 200, body = AppView)),
)]
pub async fn create(
	State(state): State<AppState>,
	Json(args): Json<CreateArgs>,
) -> Result<Json<AppView>> {
	let resolved = resolve_config(&state, args.config_branch.as_deref()).await?;
	let mut conn = state.db.get().await?;
	store_config(&mut conn, &resolved).await?;
	let app =
		Application::create_draft(&mut conn, &resolved.hash, None, &serde_json::json!({})).await?;
	Ok(Json(build_view(
		app,
		&resolved.ruleset,
		None,
		&state.default_ruleset.hash,
	)?))
}

/// Read an application with its current evaluation.
#[utoipa::path(
    post, path = "/get", operation_id = "applications_get", tag = "applications",
    request_body = GetArgs,
    responses((status = 200, body = AppView), (status = 404, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn get(
	State(state): State<AppState>,
	Json(args): Json<GetArgs>,
) -> Result<Json<AppView>> {
	let mut conn = state.db.get().await?;
	let app = Application::get(&mut conn, args.id).await?;
	let ruleset = load_ruleset(&mut conn, &app.config_hash).await?;
	Ok(Json(build_view(
		app,
		&ruleset,
		None,
		&state.default_ruleset.hash,
	)?))
}

/// Replace a draft's answers. Rejected (409) once finalised.
#[utoipa::path(
    post, path = "/patch", operation_id = "applications_patch", tag = "applications",
    request_body = PatchArgs,
    responses((status = 200, body = AppView), (status = 409, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn patch(
	State(state): State<AppState>,
	Json(args): Json<PatchArgs>,
) -> Result<Json<AppView>> {
	// Validate the answers shape before writing.
	serde_json::from_value::<Answers>(args.answers.clone())
		.map_err(|e| AppError::BadRequest(format!("invalid answers: {e}")))?;
	let mut conn = state.db.get().await?;
	let app = Application::get(&mut conn, args.id).await?;
	if app.status != ApplicationStatus::Draft {
		return Err(AppError::Conflict("application is finalised".into()));
	}
	let app = Application::set_answers(&mut conn, args.id, &args.answers).await?;
	let ruleset = load_ruleset(&mut conn, &app.config_hash).await?;
	Ok(Json(build_view(
		app,
		&ruleset,
		None,
		&state.default_ruleset.hash,
	)?))
}

/// Finalise a draft, freezing it against its bound ruleset. Always produces an
/// artifact; the verdict (including a blocking one) is recorded, not enforced.
#[utoipa::path(
    post, path = "/finalise", operation_id = "applications_finalise", tag = "applications",
    request_body = FinaliseArgs,
    responses((status = 200, body = AppView), (status = 409, body = crate::error::ProblemDetailsSchema)),
)]
pub async fn finalise(
	State(state): State<AppState>,
	Json(args): Json<FinaliseArgs>,
) -> Result<Json<AppView>> {
	let mut conn = state.db.get().await?;
	let app = Application::get(&mut conn, args.id).await?;
	if app.status != ApplicationStatus::Draft {
		return Err(AppError::Conflict(
			"application is already finalised".into(),
		));
	}
	let ruleset = load_ruleset(&mut conn, &app.config_hash).await?;
	let answers: Answers = serde_json::from_value(app.answers.clone()).map_err(AppError::custom)?;
	let evaluation = evaluate(&ruleset, &answers);
	// Every visible question must be answered; answering can reveal more, so
	// "all visible answered" means the form is complete.
	if evaluation
		.visible_questions
		.iter()
		.any(|qid| !answers.answered(qid))
	{
		return Err(AppError::BadRequest(
			"answer every question before finalising".into(),
		));
	}
	let app = Application::finalise(&mut conn, args.id).await?;
	Ok(Json(build_view(
		app,
		&ruleset,
		None,
		&state.default_ruleset.hash,
	)?))
}

/// Fork a new draft from any application, with lineage to it. `to_default`
/// rebinds to the bundled default; a named branch rebinds to that previewed
/// ruleset; otherwise the parent's ruleset is kept. Rebinding migrates the
/// answers (stable-id set-diff). The parent is left untouched.
#[utoipa::path(
    post, path = "/fork", operation_id = "applications_fork", tag = "applications",
    request_body = ForkArgs,
    responses((status = 200, body = AppView)),
)]
pub async fn fork(
	State(state): State<AppState>,
	Json(args): Json<ForkArgs>,
) -> Result<Json<AppView>> {
	let mut conn = state.db.get().await?;
	let parent = Application::get(&mut conn, args.id).await?;
	let parent_ruleset = load_ruleset(&mut conn, &parent.config_hash).await?;
	let parent_answers: Answers =
		serde_json::from_value(parent.answers.clone()).map_err(AppError::custom)?;

	// `to_default` rebinds to the bundled default; a named branch rebinds (and
	// stores) the previewed ruleset; otherwise keep the parent's binding
	// (already in config_store).
	let (new_hash, new_ruleset) = if args.to_default {
		let resolved = (*state.default_ruleset).clone();
		store_config(&mut conn, &resolved).await?;
		(resolved.hash, resolved.ruleset)
	} else {
		match args.config_branch.as_deref() {
			Some(branch) => {
				let resolved = state
					.resolver
					.as_ref()
					.ok_or_else(|| {
						AppError::BadRequest("ruleset preview is not configured".into())
					})?
					.resolve(branch)
					.await?;
				store_config(&mut conn, &resolved).await?;
				(resolved.hash, resolved.ruleset)
			}
			None => (parent.config_hash.clone(), parent_ruleset.clone()),
		}
	};

	let migration = migrate(&parent_ruleset, &new_ruleset, &parent_answers);
	let carried = serde_json::to_value(&migration.answers).map_err(AppError::custom)?;
	let app = Application::create_draft(&mut conn, &new_hash, Some(parent.id), &carried).await?;

	let view = MigrationView {
		dropped: migration.dropped,
		new_questions: migration.new_questions,
	};
	Ok(Json(build_view(
		app,
		&new_ruleset,
		Some(view),
		&state.default_ruleset.hash,
	)?))
}

pub fn routes() -> OpenApiRouter<AppState> {
	OpenApiRouter::new()
		.routes(routes!(create))
		.routes(routes!(get))
		.routes(routes!(patch))
		.routes(routes!(finalise))
		.routes(routes!(fork))
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Resolve the ruleset to bind: a named branch via the preview resolver, or the
/// bundled default.
async fn resolve_config(state: &AppState, branch: Option<&str>) -> Result<ResolvedRuleset> {
	match branch {
		Some(branch) => {
			state
				.resolver
				.as_ref()
				.ok_or_else(|| AppError::BadRequest("ruleset preview is not configured".into()))?
				.resolve(branch)
				.await
		}
		None => Ok((*state.default_ruleset).clone()),
	}
}

async fn store_config(
	conn: &mut diesel_async::AsyncPgConnection,
	resolved: &ResolvedRuleset,
) -> Result<()> {
	let content = serde_json::to_value(&resolved.ruleset).map_err(AppError::custom)?;
	ConfigRow::upsert(conn, &resolved.hash, &content).await
}

async fn load_ruleset(conn: &mut diesel_async::AsyncPgConnection, hash: &str) -> Result<Ruleset> {
	let row = ConfigRow::get(conn, hash).await?;
	serde_json::from_value(row.content).map_err(AppError::custom)
}

fn build_view(
	app: Application,
	ruleset: &Ruleset,
	migration: Option<MigrationView>,
	default_hash: &str,
) -> Result<AppView> {
	let answers: Answers = serde_json::from_value(app.answers.clone()).map_err(AppError::custom)?;
	let evaluation = evaluate(ruleset, &answers);
	// A draft bound to anything other than the current default can be updated.
	let update_available =
		app.status == ApplicationStatus::Draft && app.config_hash.as_str() != default_hash;
	Ok(AppView {
		id: app.id,
		status: app.status,
		parent_id: app.parent_id,
		created_at: app.created_at,
		finalised_at: app.finalised_at,
		config_hash: app.config_hash,
		update_available,
		questions: ruleset.questions.iter().map(QuestionView::from).collect(),
		answers: app.answers,
		evaluation,
		migration,
	})
}
