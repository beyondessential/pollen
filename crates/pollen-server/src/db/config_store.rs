use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};

/// One stored ruleset, addressed by the hash of its normalized content.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::config_store)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConfigRow {
	pub config_hash: String,
	pub content: Value,
	#[diesel(deserialize_as = jiff_diesel::Timestamp)]
	pub created_at: Timestamp,
}

impl ConfigRow {
	/// Store a normalized ruleset under its content hash. Idempotent: identical
	/// content already present is left untouched (the hash is its identity).
	pub async fn upsert(
		db: &mut AsyncPgConnection,
		config_hash: &str,
		content: &Value,
	) -> Result<()> {
		use crate::db::schema::config_store::dsl;
		diesel::insert_into(dsl::config_store)
			.values((dsl::config_hash.eq(config_hash), dsl::content.eq(content)))
			.on_conflict(dsl::config_hash)
			.do_nothing()
			.execute(db)
			.await
			.map_err(AppError::from)
			.map(|_| ())
	}

	pub async fn get(db: &mut AsyncPgConnection, config_hash: &str) -> Result<Self> {
		use crate::db::schema::config_store::dsl;
		dsl::config_store
			.find(config_hash)
			.select(Self::as_select())
			.first(db)
			.await
			.map_err(AppError::from)
	}
}
