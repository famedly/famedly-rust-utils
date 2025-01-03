//! OpenTelemetry initialization
//!
//! Lib containing the definitions and initializations of the OpenTelemetry
//! tools
use std::str::FromStr as _;

use config::{MissingConfigError, OtelConfig};
use opentelemetry::{
	trace::{TraceError, TracerProvider as _},
	KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::{
	logs::{LogError, LoggerProvider},
	metrics::{MeterProviderBuilder, MetricError, PeriodicReader, SdkMeterProvider},
	propagation::TraceContextPropagator,
	runtime,
	trace::{RandomIdGenerator, TracerProvider},
	Resource,
};
use opentelemetry_semantic_conventions::{
	resource::{SERVICE_NAME, SERVICE_VERSION},
	SCHEMA_URL,
};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{
	layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter, Layer,
};
use url::Url;

pub mod config;
mod reqwest_middleware;
pub use reqwest_middleware::OtelMiddleware;

fn resource(service_name: String, version: String) -> Resource {
	Resource::from_schema_url(
		[KeyValue::new(SERVICE_NAME, service_name), KeyValue::new(SERVICE_VERSION, version)],
		SCHEMA_URL,
	)
}

fn init_traces(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<TracerProvider, TraceError> {
	let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;
	let tracer_provider = TracerProvider::builder()
		.with_id_generator(RandomIdGenerator::default())
		.with_resource(resource(service_name, version))
		// .with_simple_exporter(exporter)
		.with_batch_exporter(exporter, runtime::Tokio)
		.build();

	opentelemetry::global::set_tracer_provider(tracer_provider.clone());
	Ok(tracer_provider)
}

fn init_metrics(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<SdkMeterProvider, MetricError> {
	let exporter = opentelemetry_otlp::MetricExporter::builder()
		.with_tonic()
		.with_endpoint(endpoint)
		.with_temporality(opentelemetry_sdk::metrics::Temporality::default())
		.build()?;

	let reader = PeriodicReader::builder(exporter, runtime::Tokio)
		// TODO: Should this be configurable or not?
		.with_interval(std::time::Duration::from_secs(1))
		.build();

	let meter_provider = MeterProviderBuilder::default()
		.with_resource(resource(service_name, version))
		.with_reader(reader)
		.build();

	Ok(meter_provider)
}

fn init_logs(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<LoggerProvider, LogError> {
	let exporter = LogExporter::builder().with_tonic().with_endpoint(endpoint).build()?;

	Ok(LoggerProvider::builder()
		.with_resource(resource(service_name, version))
		.with_batch_exporter(exporter, runtime::Tokio)
		.build())
}

/// Initializes the OpenTelemetry
#[must_use]
pub fn init_otel(config: OtelConfig) -> Result<ProvidersGuard, OtelInitError> {
	opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

	let stdout_layer = if config.stdout_enable() {
		let logger_config = config.get_stdout_config();
		let filter_fmt = EnvFilter::from_str(&logger_config.get_filter())?;

		Some(tracing_subscriber::fmt::layer().with_thread_names(true).with_filter(filter_fmt))
	} else {
		None
	};

	let (logger_provider, logs_layer) = if config.logs_enable() {
		let logger_config = config.get_logs_config()?;
		let filter_otel = EnvFilter::from_str(&logger_config.get_filter())?;
		let logger_provider =
			init_logs(config.get_endpoint(), config.get_service_name(), config.get_version())?;

		// Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
		let logs_layer = OpenTelemetryTracingBridge::new(&logger_provider);
		let logs_layer = logs_layer.with_filter(filter_otel);

		(Some(logger_provider), Some(logs_layer))
	} else {
		(None, None)
	};

	let (tracer_provider, tracer_layer) = if config.traces_enable() {
		let tracer_config = config.get_traces_config()?;

		let trace_filter = EnvFilter::from_str(&tracer_config.get_filter())?;
		let tracer_provider =
			init_traces(config.get_endpoint(), config.get_service_name(), config.get_version())?;
		let tracer = tracer_provider.tracer(config.get_service_name());
		let tracer_layer = OpenTelemetryLayer::new(tracer).with_filter(trace_filter);

		(Some(tracer_provider), Some(tracer_layer))
	} else {
		(None, None)
	};

	let (meter_provider, meter_layer) = if config.metrics_enable() {
		let meter_config = config.get_metrics_config()?;

		let metrics_filter = EnvFilter::from_str(&meter_config.get_filter())?;
		let meter_provider =
			init_metrics(config.get_endpoint(), config.get_service_name(), config.get_version())?;
		let meter_layer = MetricsLayer::new(meter_provider.clone()).with_filter(metrics_filter);

		(Some(meter_provider), Some(meter_layer))
	} else {
		(None, None)
	};

	// Initialize the tracing subscriber with the OpenTelemetry layer, the
	// stdout layer, traces and metrics.
	tracing_subscriber::registry()
		.with(logs_layer)
		.with(stdout_layer)
		.with(meter_layer)
		.with(tracer_layer)
		.init();

	Ok(ProvidersGuard { logger_provider, tracer_provider, meter_provider })
}

/// Guarding object to make sure the providers are properly shutdown
#[derive(Debug)]
pub struct ProvidersGuard {
	logger_provider: Option<LoggerProvider>,
	tracer_provider: Option<TracerProvider>,
	meter_provider: Option<SdkMeterProvider>,
}

// Necessary to call TracerProvider::shutdown() on exit
// due to a bug with flushing on global shutdown:
// https://github.com/open-telemetry/opentelemetry-rust/issues/1961
impl Drop for ProvidersGuard {
	fn drop(&mut self) {
		// This causes a hang in testing.
		// Some relevant information:
		// https://github.com/open-telemetry/opentelemetry-rust/issues/536
		#[cfg(not(test))]
		{
			self.logger_provider.as_ref().inspect(|logger_provider| {
				if let Err(err) = logger_provider.shutdown() {
					tracing::error!("Could not shutdown LoggerProvider: {err}");
				}
			});
			self.tracer_provider.as_ref().inspect(|tracer_provider| {
				if let Err(err) = tracer_provider.shutdown() {
					tracing::error!("Could not shutdown TracerProvider: {err}");
				}
			});
			self.meter_provider.as_ref().inspect(|meter_provider| {
				if let Err(err) = meter_provider.shutdown() {
					tracing::error!("Could not shutdown MeterProvider: {err}");
				}
			});
		}
	}
}

/// OpenTelemetry setup errors
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum OtelInitError {
	#[error("Logger initialization error: {0}")]
	LoggerInitError(#[from] LogError),
	#[error("Tracer initialization error: {0}")]
	TracerInitError(#[from] TraceError),
	#[error("Meter initialization error: {0}")]
	MeterInitError(#[from] MetricError),
	#[error("Parsing EnvFilter directives error: {0}")]
	EnvFilterError(#[from] tracing_subscriber::filter::ParseError),
	#[error("Otel configuration is missing: {0}")]
	MissingConfig(#[from] MissingConfigError),
}
