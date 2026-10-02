use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Droits de l'utilisateur sur un projet (et éventuellement une tâche de ce projet), en une requête.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectAccessQueryView {
    params: Vec<QueryParam>,
}

impl ProjectAccessQueryView {
    pub fn new(project_id: u64, task_id: Option<u64>, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(project_id as i32),
                QueryParam::OptionI32(task_id.map(|id| id as i32)),
            ],
        }
    }
}

impl ApiRequestDto for ProjectAccessQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT jsonb_build_object( \
                'visible', EXISTS (SELECT 1 FROM projects p WHERE p.id = $2 AND ",
            crate::project_visible_to_user_sql!(),
            "), \
                'manager_role', EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
                    WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Maire', 'Responsable')), \
                'task_exists', EXISTS (SELECT 1 FROM tasks t WHERE t.id = $3 AND t.project_id = $2), \
                'assigned', EXISTS (SELECT 1 FROM tasks t \
                    WHERE t.id = $3 AND t.project_id = $2 AND t.assigned_to = $1) \
            )"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectAccess {
    pub visible: bool,
    pub manager_role: bool,
    pub task_exists: bool,
    pub assigned: bool,
}

impl ProjectAccess {
    /// Admin, Maire ou Responsable qui voit le projet.
    pub fn can_manage(&self) -> bool {
        self.visible && self.manager_role
    }

    /// Gestionnaire du projet ou agent assigné à la tâche.
    pub fn can_act_on_task(&self) -> bool {
        self.can_manage() || (self.visible && self.assigned)
    }
}

/// Locks the project row (`FOR UPDATE`) until the end of the transaction and returns how many rows
/// were locked (0 for an unknown project).
///
/// Every write on a project, its tasks or its members takes this lock before checking access, so
/// the check and the write see the same project: a concurrent removal of the caller from the
/// members, or a deletion of the project, waits for the write to commit instead of slipping in
/// between.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockProjectQueryView {
    params: Vec<QueryParam>,
}

impl LockProjectQueryView {
    pub fn new(project_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(project_id as i32)],
        }
    }
}

impl ApiRequestDto for LockProjectQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT count(*) FROM (SELECT id FROM projects WHERE id = $1 FOR UPDATE) locked"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
