use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectView};

#[tokio::test]
async fn test_get_project_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", Some("Test Description"), 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1, 500, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        !projects.is_empty(),
        "Expected projects to be non-empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_project_none_description_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = CreateProjectQueryView::new("Test Project", None, 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );

    let view = GetProjectsQueryView::new(1, 500, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        !projects.is_empty(),
        "Expected projects to be non-empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_project_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;

    let view = GetProjectsQueryView::new(999_999, 100, 0);
    let result = db
        .fetch_one::<PagedRows<ProjectView>, _>(&view)
        .await
        .map(|page| page.items);

    assert!(
        result.is_ok(),
        "Expected result to be Ok, got: {:?}",
        result
    );
    let projects = result.unwrap();
    assert!(
        projects.is_empty(),
        "Expected projects to be empty, got: {:?}",
        projects
    );
}

#[tokio::test]
async fn test_get_projects_is_paginated_with_an_exact_total() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let owner = create_user(&db, "Pager", None).await;
    let mut ids = Vec::new();
    for title in ["A", "B", "C"] {
        ids.push(
            db.fetch_scalar::<i32, _>(&CreateProjectQueryView::new(title, None, owner))
                .await
                .unwrap(),
        );
    }

    let page: PagedRows<ProjectView> = db
        .fetch_one(&GetProjectsQueryView::new(owner, 2, 1))
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(
        page.items.iter().map(ProjectView::id).collect::<Vec<_>>(),
        vec![ids[1], ids[0]]
    );

    let past_the_end: PagedRows<ProjectView> = db
        .fetch_one(&GetProjectsQueryView::new(owner, 2, 10))
        .await
        .unwrap();
    assert_eq!(past_the_end.total, 3);
    assert!(past_the_end.items.is_empty());
}

// MAIR-474: the list carries the aggregates of the tasks and a summary of every matching project, and
// filters server side, so the BFF no longer reads every task of every visible project for one page.
mod aggregates {
    use super::*;
    use crate::common::fixtures::{fixture, unique_suffix};
    use chrono::{TimeZone, Utc};
    use mairie360_api_lib::smart_db::SmartDatabase;
    use project_api::database::project::get_projects::view::{ProjectFilters, ProjectsPage};
    use project_api::database::tasks::create_task::view::{
        CreateTaskQueryView, TaskPriority, TaskStatus,
    };
    use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;

    async fn task(
        db: &SmartDatabase,
        project: i32,
        owner: u64,
        status: TaskStatus,
        priority: TaskPriority,
        due_day: Option<u32>,
    ) {
        let due = due_day.map(|day| Utc.with_ymd_and_hms(2026, 11, day, 9, 0, 0).unwrap());
        db.fetch_scalar::<i32, _>(&CreateTaskQueryView::new(
            u64::try_from(project).unwrap(),
            owner,
            "Task",
            "",
            status,
            priority,
            due,
            None,
            &[],
        ))
        .await
        .unwrap();
    }

    /// An owner with three projects: `busy` (3 tasks, 1 completed, high priority, due 5 and 20
    /// November, 6 members), `idle` (no task, suspended) and `low` (one low task without due date).
    async fn scenario(db: &SmartDatabase) -> (u64, String, [i32; 3]) {
        let owner = create_user(db, "Aggregator", None).await;
        let tag = unique_suffix();
        let mut ids = [0; 3];
        for (i, name) in ["busy", "idle", "low"].iter().enumerate() {
            ids[i] = db
                .fetch_scalar::<i32, _>(&CreateProjectQueryView::new(
                    &format!("{name} {tag}"),
                    Some("Aggregated"),
                    owner,
                ))
                .await
                .unwrap();
        }
        task(
            db,
            ids[0],
            owner,
            TaskStatus::Completed,
            TaskPriority::Low,
            Some(20),
        )
        .await;
        task(
            db,
            ids[0],
            owner,
            TaskStatus::Todo,
            TaskPriority::High,
            Some(5),
        )
        .await;
        task(
            db,
            ids[0],
            owner,
            TaskStatus::InProgress,
            TaskPriority::Medium,
            None,
        )
        .await;
        task(db, ids[2], owner, TaskStatus::Todo, TaskPriority::Low, None).await;
        for name in ["Zoé", "Anne", "Bruno", "Chloé", "David", "Emma"] {
            let member = create_user(db, name, None).await;
            db.execute(AddUserToProjectQueryView::new(
                u64::try_from(ids[0]).unwrap(),
                member,
            ))
            .await
            .unwrap();
        }
        db.execute(fixture(format!(
            "UPDATE projects SET status = 'suspended' WHERE id = {}",
            ids[1]
        )))
        .await
        .unwrap();
        (owner, tag, ids)
    }

    async fn page(db: &SmartDatabase, owner: u64, filters: ProjectFilters) -> ProjectsPage {
        db.fetch_one(&GetProjectsQueryView::filtered(owner, &filters, 100, 0))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn every_project_carries_the_aggregates_of_its_tasks_and_its_first_members() {
        let (_container, host) = get_shared_db().await;
        let db = get_smart_db(host.to_string()).await;
        let (owner, _, ids) = scenario(&db).await;

        let result = page(&db, owner, ProjectFilters::default()).await;
        assert_eq!(result.total, 3);
        // Newest first.
        assert_eq!(
            result.items.iter().map(ProjectView::id).collect::<Vec<_>>(),
            vec![ids[2], ids[1], ids[0]]
        );
        let busy = &result.items[2];
        assert_eq!((busy.tasks_total(), busy.tasks_completed()), (3, 1));
        assert_eq!(busy.priority(), "high");
        assert_eq!(
            busy.due_date(),
            Some(Utc.with_ymd_and_hms(2026, 11, 5, 9, 0, 0).unwrap())
        );
        assert_eq!(busy.members_total(), 6);
        let names: Vec<_> = busy
            .members()
            .iter()
            .map(|member| member.name.clone().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "Anne Test",
                "Bruno Test",
                "Chloé Test",
                "David Test",
                "Emma Test"
            ]
        );

        let idle = &result.items[1];
        assert_eq!((idle.tasks_total(), idle.tasks_completed()), (0, 0));
        assert_eq!(idle.priority(), "medium", "the default priority of a task");
        assert_eq!(idle.due_date(), None);
        assert!(idle.members().is_empty());

        assert_eq!(result.by_status.active, 2);
        assert_eq!(result.by_status.suspended, 1);
        assert_eq!(
            (
                result.by_priority.low,
                result.by_priority.medium,
                result.by_priority.high
            ),
            (1, 1, 1)
        );
    }

    #[tokio::test]
    async fn the_filters_combine_and_the_summary_counts_the_matching_projects() {
        let (_container, host) = get_shared_db().await;
        let db = get_smart_db(host.to_string()).await;
        let (owner, tag, ids) = scenario(&db).await;
        let ids_of =
            |page: &ProjectsPage| page.items.iter().map(ProjectView::id).collect::<Vec<_>>();

        let search = page(
            &db,
            owner,
            ProjectFilters {
                search: Some(format!("IDLE {}", tag.to_uppercase())),
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(
            ids_of(&search),
            vec![ids[1]],
            "case-insensitive title search"
        );

        // `%` and `_` are plain characters, not LIKE wildcards.
        let wildcard = page(
            &db,
            owner,
            ProjectFilters {
                search: Some("%".to_string()),
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(wildcard.total, 0);

        let suspended = page(
            &db,
            owner,
            ProjectFilters {
                statuses: vec!["suspended"],
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(ids_of(&suspended), vec![ids[1]]);
        assert_eq!(
            (suspended.by_status.active, suspended.by_status.suspended),
            (0, 1)
        );

        let high_or_low = page(
            &db,
            owner,
            ProjectFilters {
                priorities: vec!["high", "low"],
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(ids_of(&high_or_low), vec![ids[2], ids[0]]);
        assert_eq!(high_or_low.by_priority.medium, 0);

        let due = |day| Some(Utc.with_ymd_and_hms(2026, 11, day, 0, 0, 0).unwrap());
        let before = page(
            &db,
            owner,
            ProjectFilters {
                due_before: due(10),
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(ids_of(&before), vec![ids[0]], "no due date never matches");
        let after = page(
            &db,
            owner,
            ProjectFilters {
                due_after: due(10),
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(
            after.total, 0,
            "the earliest due date (5 November) is compared"
        );

        let none = page(
            &db,
            owner,
            ProjectFilters {
                statuses: vec!["active"],
                priorities: vec!["medium"],
                ..ProjectFilters::default()
            },
        )
        .await;
        assert_eq!(none.total, 0);
        assert!(none.items.is_empty());
    }
}
