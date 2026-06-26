use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::{AppError, Result};

/// Whether an artifact is still being edited or has been frozen.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Serialize,
	Deserialize,
	diesel_derive_enum::DbEnum,
	utoipa::ToSchema,
)]
#[ExistingTypePath = "crate::db::schema::sql_types::ApplicationStatus"]
#[DbValueStyle = "snake_case"]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
	Draft,
	Finalized,
}

/// An application: a user's answers plus the ruleset hash they are bound to,
/// its draft/finalized status, and lineage back to the version it was forked
/// from.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::db::schema::applications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Application {
	pub id: Uuid,
	pub answers: Value,
	pub config_hash: String,
	pub status: ApplicationStatus,
	pub parent_id: Option<Uuid>,
	#[diesel(deserialize_as = jiff_diesel::Timestamp)]
	pub created_at: Timestamp,
	#[diesel(deserialize_as = jiff_diesel::NullableTimestamp)]
	pub finalized_at: Option<Timestamp>,
}

impl Application {
	/// Create a new draft bound to a ruleset hash, with the given starting
	/// answers. `parent_id` carries fork lineage when spawned from an existing
	/// artifact.
	pub async fn create_draft(
		db: &mut AsyncPgConnection,
		config_hash: &str,
		parent_id: Option<Uuid>,
		answers: &Value,
	) -> Result<Self> {
		use crate::db::schema::applications::dsl;
		diesel::insert_into(dsl::applications)
			.values((
				dsl::config_hash.eq(config_hash),
				dsl::parent_id.eq(parent_id),
				dsl::answers.eq(answers),
			))
			.returning(Self::as_returning())
			.get_result(db)
			.await
			.map_err(AppError::from)
	}

	pub async fn get(db: &mut AsyncPgConnection, id: Uuid) -> Result<Self> {
		use crate::db::schema::applications::dsl;
		dsl::applications
			.find(id)
			.select(Self::as_select())
			.first(db)
			.await
			.map_err(AppError::from)
	}

	/// Replace a draft's answers. Callers enforce that the application is a
	/// draft before calling.
	pub async fn set_answers(
		db: &mut AsyncPgConnection,
		id: Uuid,
		answers: &Value,
	) -> Result<Self> {
		use crate::db::schema::applications::dsl;
		diesel::update(dsl::applications.find(id))
			.set(dsl::answers.eq(answers))
			.returning(Self::as_returning())
			.get_result(db)
			.await
			.map_err(AppError::from)
	}

	/// Freeze a draft: mark it finalized and stamp the time. Callers enforce
	/// that the application is a draft before calling.
	pub async fn finalize(db: &mut AsyncPgConnection, id: Uuid) -> Result<Self> {
		use crate::db::schema::applications::dsl;
		diesel::update(dsl::applications.find(id))
			.set((
				dsl::status.eq(ApplicationStatus::Finalized),
				dsl::finalized_at.eq(jiff_diesel::NullableTimestamp::from(Some(Timestamp::now()))),
			))
			.returning(Self::as_returning())
			.get_result(db)
			.await
			.map_err(AppError::from)
	}
}
