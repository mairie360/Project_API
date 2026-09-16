use crate::endpoints::health::HealthDoc;
use crate::endpoints::hello::HelloDoc;
use crate::endpoints::v1::doc::V1Doc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

// Dans votre ApiDoc principale
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Project API — Mairie 360",
        version = "1.0.0",
        description = "\
API de gestion de projets de la plateforme **Mairie 360** : projets, tâches, champs \
personnalisés, commentaires, historique et membres. Elle ne détient ni les comptes ni les rôles, \
qui vivent dans Core API.

## Authentification

Toutes les routes sous `/api` sont protégées par `JwtMiddleware`. Le jeton s'obtient auprès de \
Core API (`POST /api/v1/auth/login`) et se présente ici dans l'en-tête `Authorization` \
(`Bearer <jwt>`).

## Droits d'accès

Chaque opération sur un projet vérifie les droits de l'appelant avant d'agir, ce qui donne la \
grille suivante :

| Exigence | Qui passe | Opérations |
| --- | --- | --- |
| Voir le projet | tout membre du projet | lectures de projet, de tâches et de membres |
| Gérer le projet | responsable du projet | création, modification, clôture, suppression, gestion des tâches et des membres |
| Agir sur la tâche | responsable du projet, ou agent assigné à la tâche | collaboration, commentaires, historique, changement de statut |

La création de projet est réservée aux rôles Admin, Maire et Responsable.

Un projet invisible pour l'appelant est traité comme **inexistant** : la réponse est `404` et non \
`403`, pour ne pas révéler son existence. Le `403` n'apparaît que sur un projet que l'appelant \
peut voir mais pas modifier.

## Format des erreurs

Les réponses d'erreur (`4xx` et `5xx`) ont un corps **`text/plain`** contenant le message \
d'erreur, et non un objet JSON.

Statuts renvoyés de façon transverse, avant même d'atteindre le handler :

| Statut | Signification |
| --- | --- |
| `400 Bad Request` | Segment d'URL qui n'est pas un entier, ou corps JSON malformé. |
| `401 Unauthorized` | En-tête `Authorization` absent, malformé, JWT invalide ou expiré, ou session révoquée. |
| `500 Internal Server Error` | Panne de la base de données ou de Redis. |
",
        contact(
            name = "Équipe Mairie 360",
            url = "https://github.com/mairie360"
        ),
        license(
            name = "Propriétaire",
            identifier = "LicenseRef-mairie360-proprietary"
        )
    ),
    servers(
        (url = "http://localhost:3001", description = "Développement local (cargo run)"),
        (url = "http://development.mairie360.fr", description = "Pile Docker de développement (nginx)")
    ),
    tags(
        (name = "Projects", description = "Cycle de vie des projets : création, consultation, modification, clôture et suppression."),
        (name = "Tasks", description = "Tâches d'un projet, leurs champs personnalisés, leurs commentaires et leur historique."),
        (name = "Users", description = "Membres d'un projet : consultation, rattachement et retrait."),
        (name = "Service", description = "Sondes techniques non authentifiées, utilisées par Docker et Kubernetes.")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = HelloDoc),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Sans ce modifier, les 16 opérations qui déclarent `security(("jwt" = []))` référencent un
/// schéma absent du contrat : Swagger UI n'offre pas de bouton « Authorize » et les clients
/// générés pointent dans le vide.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT émis par Core API (`POST /api/v1/auth/login`)."))
                    .build(),
            ),
        )
    }
}
