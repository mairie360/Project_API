use actix_web::web;
use mairie360_api_lib::state::AppState;

use crate::database::project::access::view::{ProjectAccess, ProjectAccessQueryView};

/// Refus d'accès commun aux opérations sur un projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenied {
    /// Projet invisible pour l'appelant (ou inexistant), ou tâche absente de ce projet.
    NotFound,
    /// Projet visible, mais droits insuffisants.
    Forbidden,
    DatabaseError,
}

/// Ce que l'opération exige de l'appelant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    ViewProject,
    ManageProject,
    /// Gestionnaire du projet ou agent assigné à la tâche.
    ActOnTask,
}

pub async fn require_access(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    task_id: Option<u64>,
    requirement: Requirement,
) -> Result<ProjectAccess, AccessDenied> {
    let access: ProjectAccess = state
        .get_smart_db()
        .fetch_one(&ProjectAccessQueryView::new(project_id, task_id, user_id))
        .await
        .map_err(|_| AccessDenied::DatabaseError)?;

    if !access.visible || (task_id.is_some() && !access.task_exists) {
        return Err(AccessDenied::NotFound);
    }
    let allowed = match requirement {
        Requirement::ViewProject => true,
        Requirement::ManageProject => access.can_manage(),
        Requirement::ActOnTask => access.can_act_on_task(),
    };
    if allowed {
        Ok(access)
    } else {
        Err(AccessDenied::Forbidden)
    }
}

/// Création de projet : réservée aux rôles Admin, Maire et Responsable.
pub async fn require_manager_role(
    state: &web::Data<AppState>,
    user_id: u64,
) -> Result<(), AccessDenied> {
    let access: ProjectAccess = state
        .get_smart_db()
        .fetch_one(&ProjectAccessQueryView::new(0, None, user_id))
        .await
        .map_err(|_| AccessDenied::DatabaseError)?;
    if access.manager_role {
        Ok(())
    } else {
        Err(AccessDenied::Forbidden)
    }
}
