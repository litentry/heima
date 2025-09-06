use opentelemetry::{trace::{Tracer, TracerProvider}};
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};

pub fn init_trace() -> Result<SdkTracer, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    let resource = Resource::builder()
        .with_service_name("executor-worker")
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer("executor-worker");
    Ok(tracer)
}
