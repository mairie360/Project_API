pub mod doc;

async fn not_implemented_handler() -> impl actix_web::Responder {
    actix_web::HttpResponse::NotImplemented().body("This feature is not yet implemented.")
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/templates")
            // Redirige toutes les requêtes (GET, POST, etc.) vers le handler 501
            .default_service(actix_web::web::to(not_implemented_handler)),
    );
}
