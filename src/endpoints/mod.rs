pub mod db_error;
pub mod health;
pub mod hello;
pub mod pagination;
pub mod swagger;
pub mod v1;
pub mod validation;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    // A path segment that is not a valid id answers 400 (as documented on every route) instead of
    // actix's default 404, so a client error is not mistaken for a missing row.
    cfg.app_data(
        web::PathConfig::default()
            .error_handler(|error, _| actix_web::error::ErrorBadRequest(error.to_string())),
    );
    cfg.configure(v1::config);
}
