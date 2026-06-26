use utoipa_axum::router::OpenApiRouter;

pub mod meta;

pub fn routes() -> OpenApiRouter<crate::state::AppState> {
    OpenApiRouter::new().nest("/api", OpenApiRouter::new().nest("/meta", meta::routes()))
}
