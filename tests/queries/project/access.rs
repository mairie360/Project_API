use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::project::access::view::{ProjectAccess, ProjectAccessQueryView};
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};

#[tokio::test]
async fn test_project_access_distinguishes_managers_assignees_and_outsiders() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", Some("User")).await;
    let assignee = create_user(&db, "Assignee", Some("User")).await;
    let outsider = create_user(&db, "Outsider", Some("Responsable")).await;
    let maire = create_user(&db, "Maire", Some("Maire")).await;
    let project_id = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Droits", None, owner))
        .await
        .unwrap() as u64;
    let other_project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Autre", None, owner))
        .await
        .unwrap() as u64;
    let task_id = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project_id,
            "Tâche",
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

    // L'assigné voit la tâche via son affectation, pas le projet : il n'en est ni membre ni propriétaire.
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

    assert!(owner_access.visible && owner_access.task_exists && !owner_access.can_manage());
    assert!(!owner_access.assigned && !owner_access.can_act_on_task());
    assert!(
        !assignee_access.visible && assignee_access.assigned && !assignee_access.can_act_on_task()
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
        task_exists: true,
        assigned: true,
    };
    let viewer = ProjectAccess {
        assigned: false,
        ..member
    };

    assert!(member.can_act_on_task() && !member.can_manage());
    assert!(!viewer.can_act_on_task());
}
