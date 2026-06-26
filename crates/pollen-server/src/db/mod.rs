use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::mobc::Pool;

pub mod applications;
pub mod config_store;
pub mod schema;

pub use applications::{Application, ApplicationStatus};
pub use config_store::ConfigRow;

/// Connection pool over the tool's own database.
pub type Db = Pool<AsyncPgConnection>;

pub fn init(url: &str) -> Db {
	Pool::new(AsyncDieselConnectionManager::<AsyncPgConnection>::new(url))
}
