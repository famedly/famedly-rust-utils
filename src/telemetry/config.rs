use std::str::FromStr as _;

use serde::Deserialize;
use url::Url;

use crate::LevelFilter;

const DEFAULT_FILTER: &str = "opentelemetry=off,tonic=off,h2=off,reqwest=info,axum=info,hyper=info,hyper-tls=info,tokio=info,tower=info,josekit=info,openssl=info";
const DEFAULT_LEVEL: &str = "info";
const DEFAULT_ENDPOINT: &str = "http://localhost:4317";

/// OpenTelemetry configuration
#[derive(Debug, Deserialize, Clone)]
pub struct OtelConfig {
	/// Enables logs on stdout
	pub stdout: Option<StdoutLogsConfig>,
	/// Configurations for exporting traces, metrics and logs
	pub exporter: Option<ExporterConfig>,
}

/// Configuration for exporting OpenTelemetry data
#[derive(Debug, Deserialize, Clone)]
pub struct ExporterConfig {
	/// gRPC endpoint for exporting using OTELP
	pub endpoint: Option<Url>,
	/// Application service name
	pub service_name: String,
	/// Application version
	pub version: String,

	/// Logs exporting config
	pub logger: Option<ProviderConfig>,
	/// Traces exporting config
	pub tracer: Option<ProviderConfig>,
	/// Metrics exporting config
	pub meter: Option<ProviderConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StdoutLogsConfig {
	pub enabled: bool,
	level: Option<LevelFilter>,
	filter_directives: Option<String>,
}

/// Provider configuration for OpenTelemetry export
#[derive(Debug, Deserialize, Clone)]
#[allow(missing_docs)]
pub struct ProviderConfig {
	pub enabled: bool,
	pub level: Option<LevelFilter>,
	pub filter_directives: Option<String>,
}

impl ProviderConfig {
	#[allow(clippy::expect_used)]
	pub(crate) fn get_filter(&self) -> String {
		format!(
			"{},{}",
			self.level.unwrap_or(
				LevelFilter::from_str(DEFAULT_LEVEL).expect("Error parsing default level")
			),
			self.filter_directives.as_ref().unwrap_or(&DEFAULT_FILTER.to_owned())
		)
	}
}

impl StdoutLogsConfig {
	#[allow(clippy::expect_used)]
	pub(crate) fn get_filter(&self) -> String {
		format!(
			"{},{}",
			self.level.unwrap_or(
				LevelFilter::from_str(DEFAULT_LEVEL).expect("Error parsing default level")
			),
			self.filter_directives.as_ref().unwrap_or(&DEFAULT_FILTER.to_owned())
		)
	}
}

impl Default for StdoutLogsConfig {
	fn default() -> Self {
		Self { enabled: true, level: None, filter_directives: None }
	}
}

impl ExporterConfig {
	#[allow(clippy::expect_used)]
	pub(crate) fn get_endpoint(&self) -> Url {
		self.endpoint
			.clone()
			.unwrap_or(Url::from_str(DEFAULT_ENDPOINT).expect("Error parsing default endpoint"))
	}
}
