use mairie360_api_lib::database::db_interface::{
    id_from_sql, id_to_sql, ApiRequestDto, QueryParam,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One page of the tasks of a project, oldest first. Read with `fetch_one::<PagedRows<Task>, _>`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetProjectTasksQueryView {
    params: Vec<QueryParam>,
}

impl GetProjectTasksQueryView {
    pub fn new(project_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::I64(i64::from(limit)),
                QueryParam::I64(i64::from(offset)),
            ],
        }
    }

    pub fn project_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl ApiRequestDto for GetProjectTasksQueryView {
    fn query_sql(&self) -> &'static str {
        // TIMESTAMP columns (no time zone) are converted to UTC to be read back as DateTime<Utc>.
        concat!(
            crate::paged_rows_sql!("$2", "$3"),
            " FROM ( \
                SELECT id, title, description, status, priority, created_at, assigned_to, \
                       due_date AT TIME ZONE 'UTC' AS due_date, \
                       COALESCE(custom_fields, '{}'::jsonb) AS custom_fields, \
                       row_number() OVER (ORDER BY created_at, id) AS rn \
                FROM tasks WHERE project_id = $1 \
             ) t"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Type d'un champ personnalisé de tâche : `Date`, `Checkbox` ou `Select`.
/// `Unknown` signale un type stocké en base que l'API ne sait pas interpréter.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")] // Magique : transforme "Date" en "date" dans le JSON
pub enum FieldType {
    Date,
    Checkbox,
    Select,
    #[serde(other)] // Gère les types inconnus proprement (remplace ton "Error")
    Unknown,
}

// 2. Les options du champ
/// Valeur proposée par un champ personnalisé, et son état sur la tâche.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, ToSchema)]
pub struct FieldOption {
    /// Valeur de l'option, de type libre selon le `task_type` du champ.
    #[schema(example = "Oui")]
    pub option: serde_json::Value,
    /// `true` si l'option est retenue sur cette tâche.
    #[schema(example = true)]
    pub is_selected: bool,
}

// 3. Le champ dynamique
/// Champ personnalisé défini sur le projet, avec ses valeurs pour la tâche.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, ToSchema)]
pub struct DynamicTaskField {
    /// Libellé du champ personnalisé, tel que défini sur le projet.
    #[schema(example = "Budget engagé")]
    pub label: String,
    /// Type du champ. `Unknown` signale un type stocké en base que l'API ne sait pas interpréter.
    pub task_type: FieldType,
    /// Valeurs proposées et leur état de sélection. Vide pour un champ sans options.
    pub fields_options: Vec<FieldOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: i32,
    title: String,
    #[serde(default)]
    description: String,
    status: String,
    priority: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    assigned_to: Option<i32>,
    #[serde(default)]
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    custom_fields: serde_json::Value,
}

impl Task {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn priority(&self) -> &str {
        &self.priority
    }

    pub fn created_at(&self) -> Option<&str> {
        self.created_at.as_deref()
    }

    pub fn assigned_to(&self) -> Option<i32> {
        self.assigned_to
    }

    pub fn due_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.due_date
    }

    /// Custom fields of the task. `custom_fields` holds either the ordered `fields` list (written at
    /// creation and by `PATCH`) or fields keyed by name; malformed values are skipped.
    pub fn fields(&self) -> Vec<DynamicTaskField> {
        let Some(entries) = self.custom_fields.as_object() else {
            return Vec::new();
        };
        let listed = entries
            .get("fields")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten();
        let keyed = entries
            .iter()
            .filter(|(key, _)| key.as_str() != "fields")
            .map(|(_, value)| value);
        listed
            .chain(keyed)
            .filter_map(|value| serde_json::from_value(value.clone()).ok())
            .collect()
    }
}
