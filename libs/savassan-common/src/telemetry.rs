// use opentelemetry::global;
// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace as sdktrace};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// /// Initialize OpenTelemetry tracing pipeline
// pub fn init_telemetry(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
//     // Set global propagator for context propagation via HTTP headers
//     global::set_text_map_propagator(TraceContextPropagator::new());

//     // Initialize OTLP tracer
//     let tracer = init_otlp_tracer(service_name)?;

//     // Setup tracing subscriber with OpenTelemetry layer
//     let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

//     tracing_subscriber::registry()
//         .with(telemetry)
//         .with(tracing_subscriber::EnvFilter::from_default_env())
//         .with(tracing_subscriber::fmt::layer())
//         .init();

//     Ok(())
// }

// /// Initialize OTLP tracer for exporting traces
// fn init_otlp_tracer(
//     service_name: &str,
// ) -> Result<sdktrace::Tracer, opentelemetry::trace::TraceError> {
//     let otlp_endpoint =
//         std::env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

//     opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(
//             opentelemetry_otlp::new_exporter()
//                 .tonic()
//                 .with_endpoint(otlp_endpoint),
//         )
//         .with_config(sdktrace::config().with_resource(Resource::new(vec![
//             opentelemetry::KeyValue::new("service.name", service_name.to_string()),
//             opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
//         ])))
//         .install_batch(opentelemetry_sdk::runtime::Tokio)
// }

// /// Shutdown OpenTelemetry gracefully
// pub fn shutdown_telemetry() {
//     global::shutdown_tracer_provider();
// }

// /// Create a new span for tracing
// #[macro_export]
// macro_rules! span {
//     ($name:expr) => {
//         tracing::info_span!($name)
//     };
//     ($name:expr, $($field:tt)*) => {
//         tracing::info_span!($name, $($field)*)
//     };
// }

// /// Log an event with tracing
// #[macro_export]
// macro_rules! log_event {
//     (error, $($arg:tt)*) => {
//         tracing::error!($($arg)*);
//     };
//     (warn, $($arg:tt)*) => {
//         tracing::warn!($($arg)*);
//     };
//     (info, $($arg:tt)*) => {
//         tracing::info!($($arg)*);
//     };
//     (debug, $($arg:tt)*) => {
//         tracing::debug!($($arg)*);
//     };
//     (trace, $($arg:tt)*) => {
//         tracing::trace!($($arg)*);
//     };
// }
