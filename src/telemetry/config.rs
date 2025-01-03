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
	enable: bool,
	level: Option<LevelFilter>,
	filter_directives: Option<String>,
}

/// Provider configuration for OpenTelemetry export
#[derive(Debug, Deserialize, Clone)]
#[allow(missing_docs)]
pub struct ProviderConfig {
	pub enable: bool,
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
		Self { enable: true, level: None, filter_directives: None }
	}
}

impl OtelConfig {
	pub(crate) fn stdout_enable(&self) -> bool {
		self.stdout.as_ref().is_none_or(|config| config.enable)
	}
	pub(crate) fn traces_enable(&self) -> bool {
		self.exporter
			.as_ref()
			.is_some_and(|config| config.tracer.as_ref().is_some_and(|tracer| tracer.enable))
	}
	pub(crate) fn metrics_enable(&self) -> bool {
		self.exporter
			.as_ref()
			.is_some_and(|config| config.meter.as_ref().is_some_and(|meter| meter.enable))
	}
	pub(crate) fn logs_enable(&self) -> bool {
		self.exporter
			.as_ref()
			.is_some_and(|config| config.logger.as_ref().is_some_and(|logger| logger.enable))
	}
	pub(crate) fn get_traces_config(&self) -> Result<ProviderConfig, MissingConfigError> {
		self.exporter
			.as_ref()
			.and_then(|exporter| exporter.tracer.clone())
			.ok_or(MissingConfigError::Traces)
	}
	pub(crate) fn get_metrics_config(&self) -> Result<ProviderConfig, MissingConfigError> {
		self.exporter
			.as_ref()
			.and_then(|exporter| exporter.meter.clone())
			.ok_or(MissingConfigError::Metrics)
	}
	pub(crate) fn get_logs_config(&self) -> Result<ProviderConfig, MissingConfigError> {
		self.exporter
			.as_ref()
			.and_then(|exporter| exporter.logger.clone())
			.ok_or(MissingConfigError::Logs)
	}
	pub(crate) fn get_stdout_config(&self) -> StdoutLogsConfig {
		self.stdout.clone().unwrap_or_default()
	}
	#[allow(clippy::expect_used)]
	pub(crate) fn get_endpoint(&self) -> Url {
		self.exporter
			.as_ref()
			.and_then(|exporter| exporter.clone().endpoint)
			.unwrap_or(Url::from_str(DEFAULT_ENDPOINT).expect("Error parsing default endpoint"))
	}
	pub(crate) fn get_service_name(&self) -> String {
		self.exporter
			.as_ref()
			.map_or(env!("CARGO_PKG_NAME").to_owned(), |exporter| exporter.service_name.clone())
	}
	pub(crate) fn get_version(&self) -> String {
		self.exporter
			.as_ref()
			.map_or(env!("CARGO_PKG_VERSION").to_owned(), |exporter| exporter.service_name.clone())
	}
}

/// Missing configurations errors
#[derive(Debug, thiserror::Error)]
pub enum MissingConfigError {
	#[error("Traces export configuration is missing")]
	Traces,
	#[error("Metrics export configuration is missing")]
	Metrics,
	#[error("Logs export configuration is missing")]
	Logs,
}
