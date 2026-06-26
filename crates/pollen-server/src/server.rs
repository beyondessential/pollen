use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::routing::any;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::Span;

/// Liveness/readiness probes. Stateless, so generic over the router's state.
pub fn health<S: Clone + Send + Sync + 'static>() -> Router<S> {
	Router::new()
		.route("/livez", any(async || {}))
		.route("/healthz", any(async || {}))
}

/// Wrap the application routes with the standard middleware stack.
pub fn router(routes: Router<()>) -> Router<()> {
	routes
		.layer(
			TraceLayer::new_for_http()
				.make_span_with(|request: &http::Request<_>| {
					tracing::info_span!(
						"http",
						req.uri = %request.uri(),
						req.method = %request.method(),
						res.status = tracing::field::Empty,
						latency = tracing::field::Empty,
					)
				})
				.on_response(
					|response: &http::Response<_>, latency: Duration, span: &Span| {
						span.record("latency", tracing::field::debug(latency));
						span.record(
							"res.status",
							tracing::field::display(response.status().as_u16()),
						);
						tracing::info!("response");
					},
				),
		)
		.layer(CompressionLayer::new())
}

pub async fn serve(routes: Router<()>, addr: SocketAddr) -> crate::error::Result<()> {
	let listener = TcpListener::bind(addr).await?;
	tracing::info!("listening on {}", listener.local_addr()?);
	axum::serve(listener, routes.into_make_service()).await?;
	Ok(())
}
