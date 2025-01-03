use http::Extensions;
use opentelemetry_http::HeaderInjector;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

/// Middleware for [reqwest-middleware] to propagate the Otel context
///
/// Example
///
/// let reqwest_client = reqwest::Client::builder().build().unwrap();
///	let client = ClientBuilder::new(reqwest_client)
///    // Insert the tracing middleware
///    .with(OtelMiddleware::default())
///    .build();
/// client.get("http://localhost").send().await;
#[derive(Debug, Default)]
pub struct OtelMiddleware;

#[async_trait::async_trait]
impl Middleware for OtelMiddleware {
	async fn handle(
		&self,
		mut req: Request,
		extensions: &mut Extensions,
		next: Next<'_>,
	) -> Result<Response> {
		opentelemetry::global::get_text_map_propagator(|propagator| {
			let cx = Span::current().context();
			propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut()));
		});
		next.run(req, extensions).await
	}
}
