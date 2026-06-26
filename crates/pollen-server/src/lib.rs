pub mod config;
pub mod db;
pub mod error;
pub mod fns;
pub mod openapi;
pub mod server;
pub mod spa;
pub mod state;

/// Compose the full application router: the API (under `/api`), its OpenAPI
/// document and Swagger UI, liveness probes, and the SPA fallback for
/// everything the API doesn't claim.
pub fn routes(state: state::AppState) -> error::Result<axum::Router<()>> {
	use axum::Router;
	use utoipa::OpenApi;
	use utoipa_axum::router::OpenApiRouter;
	use utoipa_swagger_ui::SwaggerUi;

	let (api_router, api_spec) = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
		.merge(fns::routes())
		.split_for_parts();

	Ok(Router::new()
		.merge(server::health())
		.merge(api_router)
		.merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", api_spec))
		.fallback(spa::handler)
		.with_state(state))
}
