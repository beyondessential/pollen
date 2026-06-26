//! Dump the OpenAPI spec to stdout as pretty-printed JSON.
//!
//! Used by `just gen-openapi` to refresh `web/openapi.json`, which the frontend
//! turns into TypeScript types via `openapi-typescript`. No database or network
//! is required — the spec is fully derived from compile-time annotations.

use pollen_server::{fns, openapi::ApiDoc};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

fn main() {
    let (_router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(fns::routes())
        .split_for_parts();
    let json = serde_json::to_string_pretty(&openapi).expect("serialize spec");
    println!("{json}");
}
