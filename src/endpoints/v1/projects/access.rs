use actix_web::web;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::db_error::log_db_error;

use crate::database::project::access::view::{
    LockProjectQueryView, ProjectAccess, ProjectAccessQueryView,
};

/// Access refusal shared by the operations on a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenied {
    /// Project the caller cannot see (or that does not exist), or task missing from this project.
    NotFound,
    /// Visible project, but insufficient rights.
    Forbidden,
    /// The caller has the rights, but the project is closed (`Completed`): it is read-only until a
    /// manager reopens it. Answered `409 Conflict`.
    Closed,
    DatabaseError,
}

/// Body of the `409` answered for a write on a closed project.
pub const PROJECT_CLOSED_MESSAGE: &str = "Project is closed.";

/// Whether a write may run on a closed project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnClosed {
    /// Refuse it with [`AccessDenied::Closed`]: the default of every write.
    Refuse,
    /// Let it run: reads, delete, close again, or a `PATCH` that may reopen the project (which
    /// then checks its body itself).
    Allow,
}

/// What the operation requires from the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    ViewProject,
    ManageProject,
    /// Manager of the project or agent assigned to the task.
    ActOnTask,
}

fn check(
    access: ProjectAccess,
    task_id: Option<u64>,
    requirement: Requirement,
    on_closed: OnClosed,
) -> Result<ProjectAccess, AccessDenied> {
    if !access.visible || (task_id.is_some() && !access.task_exists) {
        return Err(AccessDenied::NotFound);
    }
    let allowed = match requirement {
        Requirement::ViewProject => true,
        Requirement::ManageProject => access.can_manage(),
        Requirement::ActOnTask => access.can_act_on_task(),
    };
    if !allowed {
        // Rights first: a caller who may not write keeps their `403`, closed project or not.
        return Err(AccessDenied::Forbidden);
    }
    if access.closed && on_closed == OnClosed::Refuse {
        return Err(AccessDenied::Closed);
    }
    Ok(access)
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
    // A read is never refused because the project is closed.
    check(access, task_id, requirement, OnClosed::Allow)
}

/// Starts the transaction of a write on a project (or its tasks and members): locks the project
/// row, then checks the caller's rights in the same transaction.
///
/// The caller runs its queries on the returned transaction and commits it with [`commit`].
/// Dropping it (early `?` return) rolls everything back. Concurrent writes on the same project
/// are serialized, so the access check cannot be invalidated before the write (MAIR-420).
///
/// A closed project is read-only: the write is refused with [`AccessDenied::Closed`] (use
/// [`begin_write_on_closed`] for the few writes allowed on it).
pub async fn begin_write(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    task_id: Option<u64>,
    requirement: Requirement,
) -> Result<(SmartTransaction, ProjectAccess), AccessDenied> {
    begin_checked_write(
        state,
        user_id,
        project_id,
        task_id,
        requirement,
        OnClosed::Refuse,
    )
    .await
}

/// [`begin_write`] for the writes still allowed on a closed project: delete it, close it again,
/// or a `PATCH` that may reopen it. The returned access tells whether the project is closed.
pub async fn begin_write_on_closed(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    requirement: Requirement,
) -> Result<(SmartTransaction, ProjectAccess), AccessDenied> {
    begin_checked_write(
        state,
        user_id,
        project_id,
        None,
        requirement,
        OnClosed::Allow,
    )
    .await
}

async fn begin_checked_write(
    state: &web::Data<AppState>,
    user_id: u64,
    project_id: u64,
    task_id: Option<u64>,
    requirement: Requirement,
    on_closed: OnClosed,
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
    let access = check(access, task_id, requirement, on_closed)?;
    Ok((tx, access))
}

/// Commits a transaction opened by [`begin_write`]; a failure is logged under `operation`.
pub async fn commit(tx: SmartTransaction, operation: &str) -> Result<(), AccessDenied> {
    tx.commit().await.map_err(|e| {
        log_db_error(operation, &e);
        AccessDenied::DatabaseError
    })
}

/// Project creation: reserved to the Admin, Maire and Responsable roles.
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
