use utoipa_axum::router::OpenApiRouter;

pub mod applications;
pub mod meta;
pub mod pricing;

pub fn routes() -> OpenApiRouter<crate::state::AppState> {
	OpenApiRouter::new().nest(
		"/api",
		OpenApiRouter::new()
			.nest("/applications", applications::routes())
			.nest("/meta", meta::routes())
			.nest("/pricing", pricing::routes()),
	)
}
