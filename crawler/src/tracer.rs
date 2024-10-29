use opentelemetry::{
    global::{self, BoxedSpan},
    trace::{Span, TraceError, Tracer},
    KeyValue,
};
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

pub fn init_tracer_provider() -> Result<(), TraceError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .build_span_exporter()?;

    let tracer = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(
            sdktrace::Config::default().with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                "tracing-jaeger",
            )])),
        )
        .build();
    global::set_tracer_provider(tracer);
    Ok(())
}

pub fn start_span(name: &str) -> BoxedSpan {
    let tracer = global::tracer("my_service");
    let span_name = name.to_string();
    tracer.start(span_name)
}

pub fn end_span(mut span: BoxedSpan) {
    span.end();
}
