use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use project_api::database::project::access::view::{ProjectAccess, ProjectAccessQueryView};
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::update_status::view::{
    ProjectStatus, UpdateProjectStatusQueryView,
};
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

#[tokio::test]
async fn test_project_access_distinguishes_managers_assignees_and_outsiders() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", Some("User")).await;
    let assignee = create_user("Assignee", Some("User")).await;
    let outsider = create_user("Outsider", Some("Responsable")).await;
    let maire = create_user("Maire", Some("Maire")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Droits", None, owner))
        .await
        .unwrap() as u64;
    let other_project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Autre", None, owner))
        .await
        .unwrap() as u64;
    // A task can only be assigned to the owner or a member of the project (MAIR-393).
    db.execute(AddUserToProjectQueryView::new(project_id, assignee))
        .await
        .unwrap();
    let task_id = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project_id,
            1,
            "Tâche",
            "",
            TaskStatus::Todo,
            TaskPriority::Low,
            None,
            Some(assignee),
            &[],
        ))
        .await
        .unwrap() as u64;
    let access = |project: u64, task: Option<u64>, user: u64| {
        ProjectAccessQueryView::new(project, task, user)
    };

    // The assignee sees the project as a member and acts on the task through the assignment.
    let owner_access: ProjectAccess = db
        .fetch_one(&access(project_id, Some(task_id), owner))
        .await
        .unwrap();
    let assignee_access: ProjectAccess = db
        .fetch_one(&access(project_id, Some(task_id), assignee))
        .await
        .unwrap();
    let outsider_access: ProjectAccess = db
        .fetch_one(&access(project_id, None, outsider))
        .await
        .unwrap();
    let maire_access: ProjectAccess = db
        .fetch_one(&access(project_id, Some(task_id), maire))
        .await
        .unwrap();
    let wrong_project: ProjectAccess = db
        .fetch_one(&access(other_project, Some(task_id), maire))
        .await
        .unwrap();
    let role_only: ProjectAccess = db.fetch_one(&access(0, None, outsider)).await.unwrap();

    // MAIR-1088: the owner reads `manager` in its project, so it manages it whatever its role.
    assert!(owner_access.visible && owner_access.task_exists && owner_access.member_manager);
    assert!(owner_access.can_manage() && !owner_access.assigned && owner_access.can_act_on_task());
    assert!(
        assignee_access.visible
            && assignee_access.assigned
            && assignee_access.can_act_on_task()
            && !assignee_access.can_manage()
    );
    assert!(
        !outsider_access.visible && outsider_access.manager_role && !outsider_access.can_manage()
    );
    assert!(maire_access.can_manage() && maire_access.can_act_on_task());
    assert!(wrong_project.visible && !wrong_project.task_exists);
    assert!(!role_only.visible && role_only.manager_role);
}

#[test]
fn test_assigned_member_of_a_visible_project_can_act_on_its_task_only() {
    let member = ProjectAccess {
        visible: true,
        manager_role: false,
        member_manager: false,
        closed: false,
        task_exists: true,
        assigned: true,
    };
    let viewer = ProjectAccess {
        assigned: false,
        ..member
    };

    assert!(member.can_act_on_task() && !member.can_manage());
    assert!(!viewer.can_act_on_task());
    // MAIR-1088: a member with the `manager` role manages the project it sees.
    let manager_member = ProjectAccess {
        member_manager: true,
        ..viewer
    };
    assert!(manager_member.can_manage() && manager_member.can_act_on_task());
    let hidden = ProjectAccess {
        visible: false,
        ..manager_member
    };
    assert!(!hidden.can_manage());
}

/// MAIR-1088: a member whose project role is `manager` manages the project; a contributor or a
/// reader does not.
#[tokio::test]
async fn mair_1088_a_member_with_the_manager_role_manages_the_project() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", Some("Responsable")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Rôles", None, owner))
        .await
        .unwrap() as u64;
    let mut rights = Vec::new();
    for role in ["manager", "contributor", "reader"] {
        let user = create_user(role, Some("User")).await;
        db.execute(AddUserToProjectQueryView::new(project_id, user).with_role(Some(role)))
            .await
            .unwrap();
        let access: ProjectAccess = db
            .fetch_one(&ProjectAccessQueryView::new(project_id, None, user))
            .await
            .unwrap();
        rights.push((role, access.visible, access.can_manage()));
    }
    assert_eq!(
        rights,
        [
            ("manager", true, true),
            ("contributor", true, false),
            ("reader", true, false)
        ]
    );
}

/// Two writes on the same project are serialized: while a transaction holds the project lock
/// (taken by `begin_write` before the access check), another one cannot take it.
#[tokio::test]
async fn test_project_lock_serializes_writes_on_a_project() {
    use crate::common::fixtures::fixture;
    use project_api::database::project::access::view::LockProjectQueryView;

    let db = get_smart_db().await;
    let owner = create_user("Owner", Some("Responsable")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Verrou", None, owner))
        .await
        .unwrap() as u64;

    let mut first = db.begin().await.unwrap();
    let locked: i64 = first
        .fetch_scalar(&LockProjectQueryView::new(project_id))
        .await
        .unwrap();
    assert_eq!(locked, 1);

    let mut second = db.begin().await.unwrap();
    second
        .execute(&fixture("SET LOCAL lock_timeout = '200ms'".to_string()))
        .await
        .unwrap();
    assert!(
        second
            .fetch_scalar::<i64, _>(&LockProjectQueryView::new(project_id))
            .await
            .is_err(),
        "the project lock must be exclusive"
    );
    second.rollback().await.unwrap();

    first.rollback().await.unwrap();
    let mut third = db.begin().await.unwrap();
    let locked: i64 = third
        .fetch_scalar(&LockProjectQueryView::new(project_id))
        .await
        .unwrap();
    assert_eq!(locked, 1, "released at the end of the first transaction");
    third.rollback().await.unwrap();

    let mut unknown = db.begin().await.unwrap();
    let locked: i64 = unknown
        .fetch_scalar(&LockProjectQueryView::new(i32::MAX as u64))
        .await
        .unwrap();
    assert_eq!(locked, 0);
}

/// `closed` is set while the project is `completed` only, and cleared once reopened;
/// a suspended project is not closed.
#[tokio::test]
async fn mair_1200_closed_follows_the_completed_status() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", Some("Responsable")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Clôture", None, owner))
        .await
        .unwrap() as u64;
    let closed = || async {
        db.fetch_one::<ProjectAccess, _>(&ProjectAccessQueryView::new(project_id, None, owner))
            .await
            .unwrap()
            .closed
    };

    assert!(!closed().await);
    for (status, expected) in [
        (ProjectStatus::Completed, true),
        (ProjectStatus::Suspended, false),
        (ProjectStatus::Completed, true),
        (ProjectStatus::Active, false),
    ] {
        db.execute(UpdateProjectStatusQueryView::new(project_id, status))
            .await
            .unwrap();
        assert_eq!(closed().await, expected, "status {status}");
    }
}
