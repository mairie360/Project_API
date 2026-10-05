use actix_web::web;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::access::view::{
    LockProjectQueryView, ProjectAccess, ProjectAccessQueryView,
};

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

fn check(
    access: ProjectAccess,
    task_id: Option<u64>,
    requirement: Requirement,
) -> Result<ProjectAccess, AccessDenied> {
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

fn database_error(error: &impl std::fmt::Display) -> AccessDenied {
    log_db_error("projects/access", error);
    AccessDenied::DatabaseError
}

/// Access check of a read: one query, outside any transaction.
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
        .map_err(|e| database_error(&e))?;
    check(access, task_id, requirement)
}

/// Starts the transaction of a write on a project (or its tasks and members): locks the project
/// row, then checks the caller's rights in the same transaction.
///
/// The caller runs its queries on the returned transaction and commits it with [`commit`].
/// Dropping it (early `?` return) rolls everything back. Concurrent writes on the same project
/// are serialized, so the access check cannot be invalidated before the write (MAIR-420).
pub async fn begin_write(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    task_id: Option<u64>,
    requirement: Requirement,
) -> Result<(SmartTransaction, ProjectAccess), AccessDenied> {
    let mut tx = state
        .get_smart_db()
        .begin()
        .await
        .map_err(|e| database_error(&e))?;
    tx.fetch_scalar::<i64, _>(&LockProjectQueryView::new(project_id))
        .await
        .map_err(|e| database_error(&e))?;
    let access: ProjectAccess = tx
        .fetch_one(&ProjectAccessQueryView::new(project_id, task_id, user_id))
        .await
        .map_err(|e| database_error(&e))?;
    let access = check(access, task_id, requirement)?;
    Ok((tx, access))
}

/// Commits a transaction opened by [`begin_write`]; a failure is logged under `operation`.
pub async fn commit(tx: SmartTransaction, operation: &str) -> Result<(), AccessDenied> {
    tx.commit().await.map_err(|e| {
        log_db_error(operation, &e);
        AccessDenied::DatabaseError
    })
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
        .map_err(|e| database_error(&e))?;
    if access.manager_role {
        Ok(())
    } else {
        Err(AccessDenied::Forbidden)
    }
}
