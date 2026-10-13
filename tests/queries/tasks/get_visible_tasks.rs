//! `GetVisibleTasksQueryView` (MAIR-524): the tasks visible to a user across every project, in one
//! query, with their project; and the index the dashboard query relies on.

use chrono::{DateTime, TimeZone, Utc};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_projects::view::GetProjectsQueryView;
use project_api::database::tasks::create_task::view::{
    CreateTaskQueryView, TaskPriority, TaskStatus,
};
use project_api::database::tasks::get_visible_tasks::view::{
    GetVisibleTasksQueryView, TaskFilters, VisibleTaskRow,
};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

use crate::common::fixtures::{create_group, create_user, fixture, set_task_status};
use crate::common::get_smart_db;

async fn project(db: &SmartDatabase, title: &str, owner: u64) -> u64 {
    db.fetch_scalar::<i32, _>(&CreateProjectQueryView::new(title, None, owner))
        .await
        .unwrap() as u64
}

async fn task(
    db: &SmartDatabase,
    project_id: u64,
    title: &str,
    due_date: Option<DateTime<Utc>>,
    assigned_to: Option<u64>,
) -> u64 {
    let id = db
        .fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            project_id,
            1,
            title,
            "",
            TaskStatus::Todo,
            TaskPriority::Medium,
            due_date,
            assigned_to,
            &[],
        ))
        .await
        .unwrap();
    assert!(id > 0, "task {title} not created");
    id as u64
}

async fn list(db: &SmartDatabase, user: u64, filters: TaskFilters) -> PagedRows<VisibleTaskRow> {
    db.fetch_one(&GetVisibleTasksQueryView::new(user, &filters, 500, 0))
        .await
        .unwrap()
}

fn ids(page: &PagedRows<VisibleTaskRow>) -> Vec<u64> {
    page.items.iter().map(|row| row.task.id() as u64).collect()
}

fn assigned(user: u64) -> TaskFilters {
    TaskFilters {
        assigned_to: Some(user),
        ..TaskFilters::default()
    }
}

fn day(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, day, 9, 0, 0).unwrap()
}

/// A user's pending tasks of several projects come in one read, by due date (no due date last),
/// each with its project; the filters on status and due date combine with the assignee.
#[tokio::test]
async fn test_visible_tasks_span_projects_by_due_date() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", Some("Responsable")).await;
    let agent = create_user(&db, "Agent", None).await;
    let market = project(&db, "Place du marché", owner).await;
    let registry = project(&db, "État civil", owner).await;
    for project_id in [market, registry] {
        db.execute(AddUserToProjectQueryView::new(project_id, agent))
            .await
            .unwrap();
    }
    let undated = task(&db, market, "Sans échéance", None, Some(agent)).await;
    let late = task(&db, registry, "Registres", Some(day(20)), Some(agent)).await;
    let soon = task(&db, market, "Riverains", Some(day(5)), Some(agent)).await;
    let other = task(&db, market, "Pas à moi", Some(day(1)), Some(owner)).await;

    let page = list(&db, agent, assigned(agent)).await;
    assert_eq!(ids(&page), vec![soon, late, undated]);
    assert_eq!(page.total, 3);
    let first = &page.items[0];
    assert_eq!(first.project_id as u64, market);
    assert_eq!(first.project_name, "Place du marché");
    assert_eq!(first.task.due_date(), Some(day(5)));
    assert_eq!(first.task.assigned_to(), Some(agent as i32));
    assert_eq!(page.items[1].project_name, "État civil");

    // Without assignee: every visible task (the owner's one included).
    let everything = list(&db, agent, TaskFilters::default()).await;
    for id in [soon, late, undated, other] {
        assert!(ids(&everything).contains(&id), "task {id} listed");
    }

    // due_before keeps the dated tasks due at or before it.
    let due = list(
        &db,
        agent,
        TaskFilters {
            due_before: Some(day(5)),
            ..assigned(agent)
        },
    )
    .await;
    assert_eq!(ids(&due), vec![soon]);

    // A status filter.
    set_task_status(&db, late, "in_progress").await;
    let in_progress = list(
        &db,
        agent,
        TaskFilters {
            statuses: vec!["in_progress"],
            ..assigned(agent)
        },
    )
    .await;
    assert_eq!(ids(&in_progress), vec![late]);
    let todo = list(
        &db,
        agent,
        TaskFilters {
            statuses: vec!["todo"],
            ..assigned(agent)
        },
    )
    .await;
    assert_eq!(ids(&todo), vec![soon, undated]);
}

/// A completed task is archived (MAIR-502): left out by default and by every status list without
/// `completed`, listed with `archived_at` when `completed` is asked for.
#[tokio::test]
async fn test_visible_tasks_leave_archived_tasks_out_unless_completed_is_asked() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
    let project_id = project(&db, "Archives", owner).await;
    let active = task(&db, project_id, "Active", Some(day(3)), Some(owner)).await;
    let done = task(&db, project_id, "Done", Some(day(2)), Some(owner)).await;
    set_task_status(&db, done, "completed").await;

    assert_eq!(ids(&list(&db, owner, assigned(owner)).await), vec![active]);
    let not_completed = TaskFilters {
        statuses: vec!["todo", "in_progress", "other"],
        ..assigned(owner)
    };
    assert_eq!(ids(&list(&db, owner, not_completed).await), vec![active]);

    let completed = list(
        &db,
        owner,
        TaskFilters {
            statuses: vec!["completed"],
            ..assigned(owner)
        },
    )
    .await;
    assert_eq!(ids(&completed), vec![done]);
    assert!(completed.items[0].task.archived_at().is_some());

    let both = TaskFilters {
        statuses: vec!["todo", "completed"],
        ..assigned(owner)
    };
    assert_eq!(ids(&list(&db, owner, both).await), vec![done, active]);
}

/// The tasks of a project the user cannot see never appear, even with their assignee as filter;
/// Admin and Maire see them; a Responsable sees those of their team's projects.
#[tokio::test]
async fn test_visible_tasks_follow_project_visibility() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
    let teammate = create_user(&db, "Teammate", None).await;
    let responsable = create_user(&db, "Responsable", Some("Responsable")).await;
    let foreign_responsable = create_user(&db, "Foreign", Some("Responsable")).await;
    let outsider = create_user(&db, "Outsider", None).await;
    let admin = create_user(&db, "Admin", Some("Admin")).await;
    let maire = create_user(&db, "Maire", Some("Maire")).await;
    create_group(&db, owner, &[owner, responsable]).await;
    let project_id = project(&db, "Confidentiel", owner).await;
    let secret = task(&db, project_id, "Secret", Some(day(4)), Some(owner)).await;

    for (viewer, expected) in [
        (owner, true),
        (responsable, true),
        (admin, true),
        (maire, true),
        (teammate, false),
        (foreign_responsable, false),
        (outsider, false),
    ] {
        let page = list(&db, viewer, assigned(owner)).await;
        assert_eq!(ids(&page).contains(&secret), expected, "viewer {viewer}");
        if !expected {
            assert_eq!(page.total, 0, "viewer {viewer} counts nothing");
        }
        let all = list(&db, viewer, TaskFilters::default()).await;
        assert_eq!(ids(&all).contains(&secret), expected, "viewer {viewer}");
    }
}

/// `total` counts every match while the page holds `limit` tasks from `offset`, in order.
#[tokio::test]
async fn test_visible_tasks_are_paged_with_a_total() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
    let project_id = project(&db, "Pagination", owner).await;
    let mut created = Vec::new();
    for n in 1..=5 {
        created.push(
            task(
                &db,
                project_id,
                &format!("Tâche {n}"),
                Some(day(n)),
                Some(owner),
            )
            .await,
        );
    }

    let page =
        |limit, offset| GetVisibleTasksQueryView::new(owner, &assigned(owner), limit, offset);
    let second: PagedRows<VisibleTaskRow> = db.fetch_one(&page(2, 2)).await.unwrap();
    assert_eq!(second.total, 5);
    assert_eq!(ids(&second), created[2..4].to_vec());
    let past: PagedRows<VisibleTaskRow> = db.fetch_one(&page(2, 10)).await.unwrap();
    assert_eq!(past.total, 5);
    assert!(past.items.is_empty());

    // Same due date: ordered by id.
    let tie = task(&db, project_id, "Même jour", Some(day(1)), Some(owner)).await;
    let first: PagedRows<VisibleTaskRow> = db.fetch_one(&page(2, 0)).await.unwrap();
    assert_eq!(ids(&first), vec![created[0], tie]);
}

/// The SQL of a view prefixed with `EXPLAIN`, run with the view's parameters.
#[derive(serde::Serialize, serde::Deserialize)]
struct Explain {
    #[serde(skip)]
    sql: &'static str,
    params: Vec<QueryParam>,
}

impl ApiRequestDto for Explain {
    fn query_sql(&self) -> &'static str {
        self.sql
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// The plan of `view` (`EXPLAIN (FORMAT JSON)`), sequential scans disabled so that a missing
/// index shows as a `Seq Scan` instead of being hidden by the small test tables.
async fn plan<Q: ApiRequestDto>(db: &SmartDatabase, view: &Q) -> serde_json::Value {
    let explain = Explain {
        sql: Box::leak(format!("EXPLAIN (FORMAT JSON) {}", view.query_sql()).into_boxed_str()),
        params: view.query_params().to_vec(),
    };
    // Fresh statistics, so the planner sees the tasks the tests just wrote.
    db.execute(fixture("ANALYZE tasks".to_string()))
        .await
        .unwrap();
    let mut tx = db.begin().await.unwrap();
    tx.execute(&fixture("SET LOCAL enable_seqscan = off".to_string()))
        .await
        .unwrap();
    let plan: serde_json::Value = tx.fetch_one(&explain).await.unwrap();
    tx.rollback().await.unwrap();
    plan
}

/// Every scan node of `plan` on `relation`, as its node type.
fn scans_of(plan: &serde_json::Value, relation: &str, found: &mut Vec<String>) {
    match plan {
        serde_json::Value::Object(node) => {
            if node.get("Relation Name").and_then(|r| r.as_str()) == Some(relation) {
                found.push(
                    node.get("Node Type")
                        .and_then(|t| t.as_str())
                        .unwrap_or_default()
                        .to_string(),
                );
            }
            node.values()
                .for_each(|child| scans_of(child, relation, found));
        }
        serde_json::Value::Array(items) => {
            items
                .iter()
                .for_each(|child| scans_of(child, relation, found));
        }
        _ => {}
    }
}

fn assert_tasks_read_through_an_index(plan: &serde_json::Value, what: &str) {
    let mut scans = Vec::new();
    scans_of(plan, "tasks", &mut scans);
    assert!(!scans.is_empty(), "{what}: no scan of tasks in {plan}");
    for scan in &scans {
        assert!(
            matches!(
                scan.as_str(),
                "Index Scan" | "Index Only Scan" | "Bitmap Heap Scan"
            ),
            "{what}: tasks read by a {scan}, not an index: {plan}"
        );
    }
}

/// The dashboard read (`assigned_to`, `due_before`, here by an Admin, who sees every project) and
/// the projects list read `tasks` through an index (`idx_tasks_assignee_due` from Database v3.1.0, `idx_tasks_assigned_to` before it;
/// `idx_tasks_project_id` for the list aggregates), never by a sequential scan.
#[tokio::test]
async fn test_task_reads_use_an_index_on_tasks() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Owner", None).await;
    let project_id = project(&db, "Plan", owner).await;
    task(&db, project_id, "Indexée", Some(day(7)), Some(owner)).await;

    let admin = create_user(&db, "Admin", Some("Admin")).await;
    let dashboard = GetVisibleTasksQueryView::new(
        admin,
        &TaskFilters {
            due_before: Some(day(31)),
            ..assigned(owner)
        },
        20,
        0,
    );
    assert_tasks_read_through_an_index(&plan(&db, &dashboard).await, "GET /tasks?assigned_to");
    let projects = GetProjectsQueryView::new(owner, 100, 0);
    assert_tasks_read_through_an_index(&plan(&db, &projects).await, "GET /projects");
}
