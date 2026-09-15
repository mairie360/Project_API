use actix_web::{http::Method, test, web, App, HttpResponse};
use project_api::endpoints::config;
use project_api::endpoints::swagger::ApiDoc;
use utoipa::OpenApi;

// Chaque opération publiée dans le contrat OpenAPI (celui dont est généré @mairie360/project-api-openapi)
// doit correspondre à une route actix réellement montée. Aucune base ni JWT n'est nécessaire : une route
// absente tombe sur le service par défaut (418), une route trouvée échoue plus loin (données, JWT, corps).
#[actix_web::test]
async fn every_published_operation_is_routed() {
    let app = test::init_service(
        App::new()
            .service(web::scope("/api").configure(config))
            .default_service(web::to(HttpResponse::ImATeapot)),
    )
    .await;

    let document = serde_json::to_value(ApiDoc::openapi()).expect("contrat OpenAPI sérialisable");
    let paths = document["paths"].as_object().expect("paths");
    let mut unrouted = Vec::new();

    for (template, operations) in paths
        .iter()
        .filter(|(path, _)| path.starts_with("/api/v1/"))
    {
        if template.contains("//") {
            unrouted.push(format!("segment vide dans {template}"));
        }
        let uri = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    "1"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/");

        for method in operations.as_object().expect("opérations").keys() {
            let method =
                Method::from_bytes(method.to_uppercase().as_bytes()).expect("méthode HTTP");
            let request = test::TestRequest::default()
                .method(method.clone())
                .uri(&uri)
                .to_request();
            let response = test::call_service(&app, request).await;
            if response.status().as_u16() == 418 {
                unrouted.push(format!("{method} {template}"));
            }
        }
    }

    assert!(
        unrouted.is_empty(),
        "opérations publiées sans route actix : {unrouted:?}"
    );
}
