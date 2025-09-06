use opentelemetry::{trace::{Tracer, TracerProvider}};
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};

pub fn init_trace() -> Result<SdkTracer, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer("executor-worker");
    Ok(tracer)
}
