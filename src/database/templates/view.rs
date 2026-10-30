use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};

use crate::database::statuses::view::{StatusCategory, DEFAULT_STATUS_COLOR};

/// A status of a template, as written: the position is its index in the list.
#[derive(Debug, Clone)]
pub struct TemplateStatusInput<'a> {
    pub name: &'a str,
    pub color: Option<&'a str>,
    pub category: StatusCategory,
}

fn statuses_json(statuses: &[TemplateStatusInput<'_>]) -> String {
    serde_json::Value::Array(
        statuses
            .iter()
            .map(|status| {
                serde_json::json!({
                    "name": status.name,
                    "color": status.color.unwrap_or(DEFAULT_STATUS_COLOR),
                    "category": status.category.stored(),
                })
            })
            .collect(),
    )
    .to_string()
}

/// A status of a template, as read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStatusRow {
    pub name: String,
    pub color: String,
    pub category: String,
    pub position: i32,
}

/// A template with its statuses, as read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRow {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub custom_field_schema: serde_json::Value,
    pub is_default: bool,
    pub statuses: Vec<TemplateStatusRow>,
}

/// `jsonb` of one template `t` with its statuses by position.
macro_rules! template_json_sql {
    () => {
        "jsonb_build_object('id', t.id, 'name', t.name, 'description', t.description, \
            'custom_field_schema', t.custom_field_schema, 'is_default', t.is_default, \
            'statuses', COALESCE((SELECT jsonb_agg(jsonb_build_object('name', s.name, \
                'color', s.color, 'category', s.category, 'position', s.position) \
                ORDER BY s.position) FROM project_template_statuses s WHERE s.template_id = t.id), \
                '[]'::jsonb))"
    };
}

/// Every template, the default one first, then by name. Read with
/// `fetch_one::<Vec<TemplateRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTemplatesQueryView {
    params: Vec<QueryParam>,
}

impl ListTemplatesQueryView {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }
}

impl Default for ListTemplatesQueryView {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiRequestDto for ListTemplatesQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT COALESCE(jsonb_agg(",
            template_json_sql!(),
            " ORDER BY t.is_default DESC, lower(t.name), t.id), '[]'::jsonb) FROM project_templates t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// One template, `null` when it does not exist. Read with `fetch_one::<Option<TemplateRow>, _>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTemplateQueryView {
    params: Vec<QueryParam>,
}

impl GetTemplateQueryView {
    pub fn new(template_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(template_id))],
        }
    }
}

impl ApiRequestDto for GetTemplateQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT COALESCE((SELECT ",
            template_json_sql!(),
            " FROM project_templates t WHERE t.id = $1), 'null'::jsonb)"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Creates a template with its statuses (in list order) and returns its id. A name already used
/// (whatever the case) is a unique violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateQueryView {
    params: Vec<QueryParam>,
}

impl CreateTemplateQueryView {
    pub fn new(
        name: &str,
        description: Option<&str>,
        custom_field_schema: &serde_json::Value,
        statuses: &[TemplateStatusInput<'_>],
    ) -> Self {
        Self {
            params: vec![
                QueryParam::Text(name.to_string()),
                QueryParam::Text(description.unwrap_or_default().to_string()),
                QueryParam::Text(custom_field_schema.to_string()),
                QueryParam::Text(statuses_json(statuses)),
            ],
        }
    }
}

impl ApiRequestDto for CreateTemplateQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH created AS ( \
            INSERT INTO project_templates (name, description, custom_field_schema) \
            VALUES ($1, NULLIF($2, ''), $3::jsonb) RETURNING id \
         ), statuses AS ( \
            INSERT INTO project_template_statuses (template_id, name, color, category, position) \
            SELECT created.id, x.status->>'name', x.status->>'color', \
                   (x.status->>'category')::task_status, (x.ord - 1)::int \
            FROM created, jsonb_array_elements($4::jsonb) WITH ORDINALITY AS x(status, ord) \
            RETURNING 1 \
         ) \
         SELECT id FROM created"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Updates the name, description and custom fields of a template (a `None` field is kept).
/// Returns `true` when the template exists. Its statuses are replaced apart, by
/// [`ReplaceTemplateStatusesQueryView`] in the same transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateQueryView {
    params: Vec<QueryParam>,
}

impl UpdateTemplateQueryView {
    pub fn new(
        template_id: u64,
        name: Option<&str>,
        description: Option<&str>,
        custom_field_schema: Option<&serde_json::Value>,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(template_id)),
                QueryParam::Text(name.unwrap_or_default().to_string()),
                QueryParam::Bool(description.is_some()),
                QueryParam::Text(description.unwrap_or_default().to_string()),
                QueryParam::Text(
                    custom_field_schema
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                ),
            ],
        }
    }
}

impl ApiRequestDto for UpdateTemplateQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH updated AS ( \
            UPDATE project_templates SET \
                name = COALESCE(NULLIF($2, ''), name), \
                description = CASE WHEN $3 THEN NULLIF($4, '') ELSE description END, \
                custom_field_schema = COALESCE(NULLIF($5, '')::jsonb, custom_field_schema), \
                updated_at = CURRENT_TIMESTAMP \
            WHERE id = $1 RETURNING id \
         ) \
         SELECT EXISTS (SELECT 1 FROM updated)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Removes every status of a template, before [`InsertTemplateStatusesQueryView`] writes the new
/// list (two statements: a name kept from the former list must be gone before it is inserted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceTemplateStatusesQueryView {
    params: Vec<QueryParam>,
}

impl ReplaceTemplateStatusesQueryView {
    pub fn new(template_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(template_id))],
        }
    }
}

impl ApiRequestDto for ReplaceTemplateStatusesQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM project_template_statuses WHERE template_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Writes the statuses of a template, in list order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTemplateStatusesQueryView {
    params: Vec<QueryParam>,
}

impl InsertTemplateStatusesQueryView {
    pub fn new(template_id: u64, statuses: &[TemplateStatusInput<'_>]) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(template_id)),
                QueryParam::Text(statuses_json(statuses)),
            ],
        }
    }
}

impl ApiRequestDto for InsertTemplateStatusesQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO project_template_statuses (template_id, name, color, category, position) \
         SELECT $1, x.status->>'name', x.status->>'color', (x.status->>'category')::task_status, \
                (x.ord - 1)::int \
         FROM jsonb_array_elements($2::jsonb) WITH ORDINALITY AS x(status, ord)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// [`DeleteTemplateQueryView`]: the template was deleted.
pub const TEMPLATE_DELETED: i32 = 1;
/// No such template.
pub const TEMPLATE_NOT_FOUND: i32 = 0;
/// The default template cannot be deleted.
pub const TEMPLATE_IS_DEFAULT: i32 = -1;

/// Deletes a template (its projects keep their statuses, `template_id` becomes `NULL`). Returns
/// [`TEMPLATE_DELETED`], [`TEMPLATE_NOT_FOUND`] or [`TEMPLATE_IS_DEFAULT`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTemplateQueryView {
    params: Vec<QueryParam>,
}

impl DeleteTemplateQueryView {
    pub fn new(template_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(template_id))],
        }
    }
}

impl ApiRequestDto for DeleteTemplateQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH found AS (SELECT id, is_default FROM project_templates WHERE id = $1), \
         deleted AS ( \
            DELETE FROM project_templates t USING found \
            WHERE t.id = found.id AND NOT found.is_default RETURNING t.id \
         ) \
         SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM found) THEN 0 \
                     WHEN EXISTS (SELECT 1 FROM deleted) THEN 1 ELSE -1 END"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// `true` when the user may manage templates: Admin or Responsable (MAIR-566).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManagerQueryView {
    params: Vec<QueryParam>,
}

impl TemplateManagerQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id))],
        }
    }
}

impl ApiRequestDto for TemplateManagerQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
                        WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Responsable'))"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
