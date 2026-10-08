//! OpenTelemetry tracing (MAIR-503): each request yields a root span carrying the route and the
//! status, continues an incoming `traceparent`, carries the SQL it ran as span events, and leaves
//! without the client address nor the query string.

use std::io::Write;
use std::sync::{Arc, Mutex};

use actix_web::test::TestRequest;
use actix_web::{middleware, App, HttpResponse};
use opentelemetry::global;
use opentelemetry::Value;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{
    InMemorySpanExporter, SdkTracerProvider, SimpleSpanProcessor, SpanData,
};
use project_api::telemetry::{log_layer, trace_layer, tracer_provider};
use serial_test::serial;
use tracing::subscriber::DefaultGuard;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

use super::{get, TestContext};

const PROJECTS: &str = "/api/v1/projects/";
const PROJECTS_ROUTE: &str = "GET /api/v1/projects/";

/// Same mounting as `main.rs`, `TracingLogger` around the `/api` scope.
macro_rules! init_traced_app {
    ($ctx:expr) => {
        actix_web::test::init_service(
            App::new()
                .wrap(TracingLogger::default())
                .app_data($ctx.state.clone())
                .service(
                    actix_web::web::scope("/api")
                        .wrap(mairie360_api_lib::security::JwtMiddleware)
                        .configure(project_api::endpoints::config),
                ),
        )
        .await
    };
}

/// Provider exporting to memory through the redaction of `telemetry`, and the subscriber feeding
/// it, installed for the current thread.
fn in_memory_tracing() -> (InMemorySpanExporter, SdkTracerProvider, DefaultGuard) {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = tracer_provider(SimpleSpanProcessor::new(exporter.clone()));
    let guard = tracing::subscriber::set_default(
        tracing_subscriber::registry().with(trace_layer(&provider)),
    );
    (exporter, provider, guard)
}

fn request_span(exporter: &InMemorySpanExporter, provider: &SdkTracerProvider) -> SpanData {
    provider.force_flush().expect("flush the spans");
    let spans = exporter.get_finished_spans().unwrap();
    spans
        .iter()
        .find(|span| span.name == PROJECTS_ROUTE)
        .cloned()
        .unwrap_or_else(|| panic!("no span named {PROJECTS_ROUTE}: {spans:#?}"))
}

fn attribute<'a>(span: &'a SpanData, key: &str) -> Option<&'a Value> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| &kv.value)
}

#[actix_web::test]
#[serial]
async fn a_request_is_exported_as_a_span_with_its_sql() {
    let (exporter, provider, _guard) = in_memory_tracing();
    let ctx = TestContext::new().await;
    let user = ctx.user("telemetry_endpoint", Some("Admin")).await;
    let app = init_traced_app!(ctx);
    exporter.reset();

    let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let request = get(&format!("{PROJECTS}?limit=5&offset=0"), user)
        .insert_header(("traceparent", format!("00-{trace_id}-00f067aa0ba902b7-01")))
        .to_request();
    let response = actix_web::test::call_service(&app, request).await;
    assert!(response.status().is_success(), "{}", response.status());
    // The root span closes with the response body: drop it before reading the exported spans.
    drop(response);

    let span = request_span(&exporter, &provider);
    assert_eq!(
        span.span_context.trace_id().to_string(),
        trace_id,
        "the incoming traceparent must be continued"
    );
    assert_eq!(attribute(&span, "http.status_code"), Some(&Value::I64(200)));
    assert_eq!(attribute(&span, "http.route"), Some(&Value::from(PROJECTS)));
    assert_eq!(
        attribute(&span, "http.target"),
        Some(&Value::from(PROJECTS)),
        "the query string must not be exported"
    );
    assert_eq!(attribute(&span, "http.client_ip"), None);
    assert_eq!(
        attribute(&span, "service.name"),
        None,
        "service.name belongs to the resource"
    );

    // `sqlx` logs each statement as a `sqlx::query` event; the values stay bound parameters.
    let has_sql = span.events.iter().any(|event| {
        let has = |key: &str| event.attributes.iter().any(|kv| kv.key.as_str() == key);
        has("db.statement")
            && event
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "target" && kv.value == Value::from("sqlx::query"))
    });
    assert!(
        has_sql,
        "the SQL run by the handler must be attached to its span: {:#?}",
        span.events
    );
}

#[actix_web::test]
#[serial]
async fn a_refused_request_still_gets_its_span() {
    let (exporter, provider, _guard) = in_memory_tracing();
    let ctx = TestContext::new().await;
    let app = init_traced_app!(ctx);
    exporter.reset();

    // No JWT: `JwtMiddleware` refuses before the handler runs.
    let _ = actix_web::test::try_call_service(&app, TestRequest::get().uri(PROJECTS).to_request())
        .await;

    let span = request_span(&exporter, &provider);
    assert_eq!(attribute(&span, "http.status_code"), Some(&Value::I64(401)));
}

/// Collects the stdout logs of [`log_layer`].
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for Captured {
    type Writer = Self;

    fn make_writer(&'writer self) -> Self::Writer {
        self.clone()
    }
}

#[actix_web::get("/api/v1/projects/")]
async fn logging_handler() -> HttpResponse {
    tracing::error!("the handler logs an error");
    HttpResponse::Ok().finish()
}

/// No database: the logs and spans of a request carrying personal data in its query string and
/// client address, mounted like `main.rs` (`Logger`, then `TracingLogger`).
#[actix_web::test]
#[serial]
async fn neither_logs_nor_spans_carry_the_query_string_or_the_client_address() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = tracer_provider(SimpleSpanProcessor::new(exporter.clone()));
    let logs = Captured::default();
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(log_layer(logs.clone()))
            .with(trace_layer(&provider)),
    );
    let app = actix_web::test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(TracingLogger::default())
            .service(logging_handler),
    )
    .await;

    let request = TestRequest::get()
        .uri("/api/v1/projects/?search=Dupont")
        .insert_header(("x-forwarded-for", "203.0.113.7"))
        .to_request();
    let response = actix_web::test::call_service(&app, request).await;
    drop(response);
    provider.force_flush().expect("flush the spans");

    let logs = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    let event = logs
        .lines()
        .find(|line| line.contains("the handler logs an error"))
        .unwrap_or_else(|| panic!("the handler's event must be logged: {logs}"));
    assert!(
        !event.contains("Dupont"),
        "query string in the logs: {event}"
    );
    assert!(
        !event.contains("203.0.113.7"),
        "client address in the logs: {event}"
    );

    let spans = format!("{:?}", exporter.get_finished_spans().unwrap());
    assert!(spans.contains("the handler logs an error"), "{spans}");
    assert!(
        !spans.contains("Dupont"),
        "query string in the spans: {spans}"
    );
    assert!(
        !spans.contains("203.0.113.7"),
        "client address in the spans: {spans}"
    );
}
