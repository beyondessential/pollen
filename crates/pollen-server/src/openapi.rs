use utoipa::OpenApi;

/// Base OpenAPI document. Concrete paths and schemas are pulled in by
/// `utoipa-axum`'s `OpenApiRouter` as routes are registered. The tool is
/// unauthenticated, so there are no security schemes.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "pollen",
        version = "",
        description = "Public API for the Tamanu deployment onboarding wizard.",
        contact(name = "BES Developers", email = "contact@bes.au"),
        license(name = "GPL-3.0-or-later"),
    ),
    tags(
        (name = "applications", description = "Onboarding artifacts: create, read, edit, finalize, fork."),
        (name = "meta", description = "Service metadata."),
    ),
)]
pub struct ApiDoc;
