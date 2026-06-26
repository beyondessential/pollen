//! Shared test support: a throwaway per-test database on the cluster
//! `DATABASE_URL` points at (the ramdisk Postgres under `just test`), and an
//! HTTP harness that mounts the app against one.
//!
//! Shared across test binaries, each of which uses a different subset.
#![allow(dead_code)]

use std::sync::Arc;

use axum_test::TestServer;
use diesel_async::{
	AsyncConnection as _, AsyncMigrationHarness, AsyncPgConnection, SimpleAsyncConnection as _,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness as _, embed_migrations};
use pollen_server::config::Config;
use pollen_server::ruleset::{BUNDLED_RULESET, ResolvedRuleset};
use pollen_server::state::AppState;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct TestDb {
	name: String,
	url: String,
	admin_url: String,
}

impl TestDb {
	async fn init() -> Self {
		let base = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
		// Swap the database-name path component for a fresh per-test name.
		let (prefix, _db) = base.rsplit_once('/').expect("DATABASE_URL has a path");
		let name = format!("pollen_test_{}", Uuid::new_v4().simple());

		let this = Self {
			url: format!("{prefix}/{name}"),
			admin_url: format!("{prefix}/postgres"),
			name,
		};

		this.connect(true)
			.await
			.batch_execute(&format!("CREATE DATABASE \"{}\";", this.name))
			.await
			.expect("create test database");

		let mut migrator = AsyncMigrationHarness::new(this.connect(false).await);
		migrator
			.run_pending_migrations(MIGRATIONS)
			.expect("run migrations");

		this
	}

	async fn connect(&self, admin: bool) -> AsyncPgConnection {
		let url = if admin { &self.admin_url } else { &self.url };
		AsyncPgConnection::establish(url)
			.await
			.expect("connect to database")
	}

	async fn teardown(self) {
		let _ = self
			.connect(true)
			.await
			.batch_execute(&format!("DROP DATABASE IF EXISTS \"{}\";", self.name))
			.await;
	}

	/// Run a test against a fresh, migrated, throwaway database.
	pub async fn run<F, T, Fut>(test: F) -> T
	where
		F: FnOnce(AsyncPgConnection) -> Fut,
		Fut: Future<Output = T>,
	{
		let tdb = TestDb::init().await;
		let result = test(tdb.connect(false).await).await;
		tdb.teardown().await;
		result
	}
}

/// Run an HTTP test against a fresh database with the app mounted: the bundled
/// ruleset is loaded and previews are disabled (no source repo configured). The
/// test gets the server and a direct connection for assertions.
pub async fn run_server<F, T, Fut>(test: F) -> T
where
	F: FnOnce(TestServer, AsyncPgConnection) -> Fut,
	Fut: Future<Output = T>,
{
	let tdb = TestDb::init().await;
	let state = AppState {
		config: Arc::new(Config {
			public_base_url: None,
			database_url: tdb.url.clone(),
			ruleset_repo: None,
			ruleset_repo_token: None,
		}),
		db: pollen_server::db::init(&tdb.url),
		default_ruleset: Arc::new(
			ResolvedRuleset::from_ron(BUNDLED_RULESET).expect("bundled ruleset"),
		),
		resolver: None,
	};
	let app = pollen_server::routes(state).expect("routes");
	let server = TestServer::new(app);

	let result = test(server, tdb.connect(false).await).await;
	tdb.teardown().await;
	result
}
