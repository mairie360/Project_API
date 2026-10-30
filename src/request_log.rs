//! Request logging without personal data (MAIR-290).
//!
//! actix's `Logger::default()` writes `%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T`: `%r` is
//! the request line, query string included (`GET /api/v1/...?search=dupont`), and the `Referer`
//! carries the query of the front page that made the call. [`request_logger`] logs the method,
//! the path without the query, the status, the size and the duration only.
//!
//! Rule: a log describes a request by its type and context, never by the value it received.

use actix_web::middleware::Logger;

/// The format of [`request_logger`]: client address, method, path (`%U`, without the query
/// string), status, body size, duration in seconds.
pub const REQUEST_LOG_FORMAT: &str = r#"%a "%{method}xi %U" %s %b %T"#;

/// actix's `Logger` with [`REQUEST_LOG_FORMAT`]: wrap the app with it instead of
/// `Logger::default()`.
#[must_use]
pub fn request_logger() -> Logger {
    Logger::new(REQUEST_LOG_FORMAT).custom_request_replace("method", |req| req.method().to_string())
}

#[cfg(test)]
mod tests {
    use super::request_logger;
    use actix_web::test as actix_test;
    use actix_web::{get, App, HttpRequest, HttpResponse};
    use std::sync::{Mutex, OnceLock};

    /// Collects what actix's `Logger` writes through the `log` facade.
    struct Captured(Mutex<Vec<String>>);

    impl log::Log for Captured {
        fn enabled(&self, _: &log::Metadata<'_>) -> bool {
            true
        }
        fn log(&self, record: &log::Record<'_>) {
            self.0.lock().unwrap().push(record.args().to_string());
        }
        fn flush(&self) {}
    }

    fn captured() -> &'static Captured {
        static CAPTURED: OnceLock<&'static Captured> = OnceLock::new();
        CAPTURED.get_or_init(|| {
            let captured: &'static Captured = Box::leak(Box::new(Captured(Mutex::new(Vec::new()))));
            log::set_logger(captured).expect("no other logger in the unit tests");
            log::set_max_level(log::LevelFilter::Info);
            captured
        })
    }

    #[get("/search")]
    async fn search(req: HttpRequest) -> HttpResponse {
        HttpResponse::Ok().body(req.query_string().to_string())
    }

    #[actix_web::test]
    async fn the_request_log_holds_no_query_string() {
        let captured = captured();
        let app = actix_test::init_service(App::new().wrap(request_logger()).service(search)).await;
        let req = actix_test::TestRequest::get()
            .uri("/search?search=gdpr.marker%40example.com")
            .insert_header(("Referer", "http://front/users?search=gdpr.marker"))
            .to_request();
        let body = actix_test::call_and_read_body(&app, req).await;
        assert_eq!(
            body, "search=gdpr.marker%40example.com",
            "the handler still reads the query"
        );

        let lines = captured.0.lock().unwrap().join("\n");
        assert!(lines.contains("\"GET /search\" 200"), "{lines}");
        assert!(!lines.contains("gdpr.marker"), "{lines}");
    }
}
