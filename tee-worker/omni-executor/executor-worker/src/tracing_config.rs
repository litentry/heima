use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use opentelemetry_sdk::Resource;

pub fn init_trace(
	port: u16,
) -> Result<SdkTracer, Box<dyn std::error::Error + Send + Sync + 'static>> {
	let endpoint = format!("http://localhost:{}", port);
	let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;

	let resource = Resource::builder().with_service_name("executor-worker").build();

	let tracer_provider = SdkTracerProvider::builder()
		.with_resource(resource)
		.with_batch_exporter(exporter)
		.build();

	let tracer = tracer_provider.tracer("executor-worker");
	Ok(tracer)
}
