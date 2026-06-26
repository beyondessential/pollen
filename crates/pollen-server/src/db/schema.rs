// @generated automatically by Diesel CLI.

pub mod sql_types {
	#[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "application_status"))]
	pub struct ApplicationStatus;
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::ApplicationStatus;

	applications (id) {
		id -> Uuid,
		answers -> Jsonb,
		config_hash -> Text,
		status -> ApplicationStatus,
		parent_id -> Nullable<Uuid>,
		created_at -> Timestamptz,
		finalised_at -> Nullable<Timestamptz>,
	}
}

diesel::table! {
	config_store (config_hash) {
		config_hash -> Text,
		content -> Jsonb,
		created_at -> Timestamptz,
	}
}

diesel::joinable!(applications -> config_store (config_hash));

diesel::allow_tables_to_appear_in_same_query!(applications, config_store,);
