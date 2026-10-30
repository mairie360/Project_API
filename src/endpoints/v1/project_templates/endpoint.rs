use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use mairie360_api_lib::database::db_interface::id_from_sql;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::templates::view::{
    CreateTemplateQueryView, DeleteTemplateQueryView, GetTemplateQueryView,
    InsertTemplateStatusesQueryView, ListTemplatesQueryView, ReplaceTemplateStatusesQueryView,
    TemplateManagerQueryView, TemplateRow, TemplateStatusInput, UpdateTemplateQueryView,
    TEMPLATE_DELETED, TEMPLATE_IS_DEFAULT,
};
use crate::endpoints::db_error::{classify_db_error, DbFailure};
use crate::endpoints::v1::project_templates::view::{
    CreateTemplateResultView, CreateTemplateView, GetTemplatesResultView, PatchTemplateView,
    ProjectTemplateView,
};
use crate::endpoints::v1::project_templates::{TemplatePathParams, TemplatesError};
use crate::endpoints::validation::ValidatedJson;

fn database_error(operation: &str, error: &ApiLibError) -> TemplatesError {
    match classify_db_error(operation, error) {
        DbFailure::Conflict => TemplatesError::NameTaken,
        _ => TemplatesError::DatabaseError,
    }
}

/// Template management is reserved to Admin and Responsable (MAIR-566).
async fn require_template_manager(
    state: &web::Data<AppState>,
    user_id: u64,
    operation: &str,
) -> Result<(), TemplatesError> {
    let allowed: bool = state
        .get_smart_db()
        .fetch_scalar(&TemplateManagerQueryView::new(user_id))
        .await
        .map_err(|e| database_error(operation, &e))?;
    if allowed {
        Ok(())
    } else {
        Err(TemplatesError::Forbidden)
    }
}

#[utoipa::path(
    get,
    path = "/",
    summary = "List the project templates",
    description = "Returns every project template, the default one first, then by name, with its \
                   task statuses and custom fields (MAIR-566). A project created with \
                   `template_id` gets a copy of them. Any signed-in user may read them.",
    responses(
        (status = 200, description = "Every template.", body = GetTemplatesResultView,
         example = json!({ "templates": [{
             "id": 1, "name": "Simple", "description": "À faire, en cours, terminé.", "is_default": true,
             "statuses": [
                 { "name": "À faire", "color": "#64748B", "category": "Todo", "position": 0 },
                 { "name": "En cours", "color": "#2563EB", "category": "InProgress", "position": 1 },
                 { "name": "Terminé", "color": "#16A34A", "category": "Completed", "position": 2 }
             ],
             "custom_fields": []
         }] })),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Templates",
)]
#[get("/")]
pub async fn get_templates(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<impl Responder, TemplatesError> {
    let rows: Vec<TemplateRow> = state
        .get_smart_db()
        .fetch_one(&ListTemplatesQueryView::new())
        .await
        .map_err(|e| database_error("project-templates/get", &e))?;
    Ok(HttpResponse::Ok().json(GetTemplatesResultView {
        templates: rows.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(
    get,
    params(TemplatePathParams),
    path = "/{template_id}/",
    summary = "Read a project template",
    description = "Returns one template with its task statuses and custom fields. Any signed-in \
                   user may read it.",
    responses(
        (status = 200, description = "The template.", body = ProjectTemplateView),
        (status = 400, description = "A URL segment is not an integer.", body = String, content_type = "text/plain",
         example = json!("Path deserialize error: can not parse `abc` to a u64")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 404, description = "No such template.", body = String, content_type = "text/plain",
         example = json!("Unknown template.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Templates",
)]
#[get("/{template_id}/")]
pub async fn get_template(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    params: web::Path<TemplatePathParams>,
) -> Result<impl Responder, TemplatesError> {
    let row: Option<TemplateRow> = state
        .get_smart_db()
        .fetch_one(&GetTemplateQueryView::new(params.template_id()))
        .await
        .map_err(|e| database_error("project-templates/template_id/get", &e))?;
    let template: ProjectTemplateView = row.ok_or(TemplatesError::NotFound)?.into();
    Ok(HttpResponse::Ok().json(template))
}

#[utoipa::path(
    post,
    path = "/",
    summary = "Create a project template",
    description = "Creates a template with its task statuses, in order, and its custom fields. \
                   Reserved to the Admin and Responsable roles. The statuses must cover the three \
                   categories (`Todo`, `InProgress`, `Completed`), so that a task of a project \
                   created from it can always be started and completed.",
    request_body(content = CreateTemplateView, example = json!({
        "name": "Travaux de voirie",
        "description": "Étude, chantier, réception.",
        "statuses": [
            { "name": "Étude", "category": "Todo" },
            { "name": "Chantier", "color": "#D97706", "category": "InProgress" },
            { "name": "Réception", "color": "#16A34A", "category": "Completed" }
        ],
        "custom_fields": []
    })),
    responses(
        (status = 200, description = "Template created.", body = CreateTemplateResultView,
         example = json!({ "template_id": 5 })),
        (status = 400, description = "Malformed JSON body, `category` not one of the enum values, or a field breaking its rules: `name` and each status name 1 to 100 characters, not blank, no control character; `description` at most 5000 characters; `color` `#` followed by 6 hexadecimal digits; 1 to 30 statuses with distinct names covering the three categories; each `custom_fields[].label` 1 to 255 characters.",
         body = String, content_type = "text/plain", example = json!("Invalid `statuses`: must hold a status of each category, none is Completed")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "The caller is neither Admin nor Responsable.", body = String,
         content_type = "text/plain", example = json!("Forbidden.")),
        (status = 409, description = "Another template already has this name.", body = String,
         content_type = "text/plain", example = json!("Another template already has this name.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Templates",
)]
#[post("/")]
pub async fn create_template(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<CreateTemplateView>,
) -> Result<impl Responder, TemplatesError> {
    const OPERATION: &str = "project-templates/post";
    require_template_manager(&state, auth_user.id, OPERATION).await?;
    let view = view.into_inner();
    let statuses: Vec<TemplateStatusInput<'_>> = view
        .statuses
        .iter()
        .map(|status| status.as_input())
        .collect();
    let template_id: i32 = state
        .get_smart_db()
        .fetch_scalar(&CreateTemplateQueryView::new(
            view.name.trim(),
            view.description.as_deref(),
            &serde_json::json!(view.custom_fields),
            &statuses,
        ))
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    Ok(HttpResponse::Ok().json(CreateTemplateResultView {
        template_id: id_from_sql(template_id),
    }))
}

#[utoipa::path(
    patch,
    params(TemplatePathParams),
    path = "/{template_id}/",
    summary = "Update a project template",
    description = "Partially updates a template: an absent field is left unchanged; `statuses` \
                   and `custom_fields` replace the whole list. Reserved to the Admin and \
                   Responsable roles. The projects already created from the template keep their \
                   statuses. The response has an empty body.",
    request_body(content = PatchTemplateView, example = json!({ "name": "Voirie" })),
    responses(
        (status = 204, description = "Template updated. Empty body."),
        (status = 400, description = "A URL segment is not an integer, malformed JSON body, or a field breaking the rules of the creation.",
         body = String, content_type = "text/plain", example = json!("Invalid `statuses`: two statuses have the same name")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "The caller is neither Admin nor Responsable.", body = String,
         content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "No such template.", body = String, content_type = "text/plain",
         example = json!("Unknown template.")),
        (status = 409, description = "Another template already has this name.", body = String,
         content_type = "text/plain", example = json!("Another template already has this name.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Templates",
)]
#[patch("/{template_id}/")]
pub async fn update_template(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TemplatePathParams>,
    view: ValidatedJson<PatchTemplateView>,
) -> Result<impl Responder, TemplatesError> {
    const OPERATION: &str = "project-templates/template_id/patch";
    require_template_manager(&state, auth_user.id, OPERATION).await?;
    let view = view.into_inner();
    let template_id = params.template_id();
    let mut tx = state
        .get_smart_db()
        .begin()
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    let custom_fields = view
        .custom_fields
        .as_ref()
        .map(|fields| serde_json::json!(fields));
    let found: bool = tx
        .fetch_scalar(&UpdateTemplateQueryView::new(
            template_id,
            view.name.as_deref().map(str::trim),
            view.description.as_deref(),
            custom_fields.as_ref(),
        ))
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    if !found {
        return Err(TemplatesError::NotFound);
    }
    if let Some(statuses) = &view.statuses {
        let statuses: Vec<TemplateStatusInput<'_>> =
            statuses.iter().map(|status| status.as_input()).collect();
        tx.execute(&ReplaceTemplateStatusesQueryView::new(template_id))
            .await
            .map_err(|e| database_error(OPERATION, &e))?;
        tx.execute(&InsertTemplateStatusesQueryView::new(
            template_id,
            &statuses,
        ))
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    }
    tx.commit()
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    params(TemplatePathParams),
    path = "/{template_id}/",
    summary = "Delete a project template",
    description = "Deletes a template. Reserved to the Admin and Responsable roles. The projects \
                   created from it keep their statuses. The default template cannot be deleted. \
                   The response has an empty body.",
    responses(
        (status = 204, description = "Template deleted. Empty body."),
        (status = 400, description = "A URL segment is not an integer.", body = String, content_type = "text/plain",
         example = json!("Path deserialize error: can not parse `abc` to a u64")),
        (status = 401, description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
         body = String, content_type = "text/plain", example = json!("Jeton expiré")),
        (status = 403, description = "The caller is neither Admin nor Responsable.", body = String,
         content_type = "text/plain", example = json!("Forbidden.")),
        (status = 404, description = "No such template.", body = String, content_type = "text/plain",
         example = json!("Unknown template.")),
        (status = 409, description = "The default template cannot be deleted.", body = String,
         content_type = "text/plain", example = json!("The default template cannot be deleted.")),
        (status = 500, description = "Database error.", body = String, content_type = "text/plain",
         example = json!("An error occurred while accessing the database.")),
    ),
    security(("jwt" = [])),
    tag = "Templates",
)]
#[delete("/{template_id}/")]
pub async fn delete_template(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<TemplatePathParams>,
) -> Result<impl Responder, TemplatesError> {
    const OPERATION: &str = "project-templates/template_id/delete";
    require_template_manager(&state, auth_user.id, OPERATION).await?;
    let result: i32 = state
        .get_smart_db()
        .fetch_scalar(&DeleteTemplateQueryView::new(params.template_id()))
        .await
        .map_err(|e| database_error(OPERATION, &e))?;
    match result {
        TEMPLATE_DELETED => Ok(HttpResponse::NoContent().finish()),
        TEMPLATE_IS_DEFAULT => Err(TemplatesError::DefaultTemplate),
        _ => Err(TemplatesError::NotFound),
    }
}
