//! MAIR-1088: the role of a project member, written at the addition and changed afterwards; the
//! owner always reads `manager`.

use crate::common::fixtures::create_user;
use crate::common::get_smart_db;
use project_api::database::paged::PagedRows;
use project_api::database::project::create::view::CreateProjectQueryView;
use project_api::database::project::get_projects::view::{GetProjectsQueryView, ProjectsPage};
use project_api::database::users::add_user_to_project::view::AddUserToProjectQueryView;
use project_api::database::users::get_project_users::view::{
    GetProjectUsersQueryView, ProjectMemberRow,
};
use project_api::database::users::member_role::view::{
    IsProjectOwnerQueryView, SetMemberRoleQueryView,
};

fn role_of(rows: &[ProjectMemberRow], user: u64) -> String {
    rows.iter()
        .find(|row| row.id as u64 == user)
        .map(|row| row.role.clone())
        .expect("member listed")
}

#[tokio::test]
async fn mair_1088_members_read_their_role_and_the_owner_reads_manager() {
    let db = get_smart_db().await;
    let owner = create_user("Owner", None).await;
    let contributor = create_user("Contributor", None).await;
    let reader = create_user("Reader", None).await;
    let project = db
        .fetch_scalar::<i32, _>(&CreateProjectQueryView::new("Médiathèque", None, owner))
        .await
        .unwrap() as u64;
    for (user, role) in [(owner, None), (contributor, None), (reader, Some("reader"))] {
        db.execute(AddUserToProjectQueryView::new(project, user).with_role(role))
            .await
            .unwrap();
    }
    let page: PagedRows<ProjectMemberRow> = db
        .fetch_one(&GetProjectUsersQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(role_of(&page.items, owner), "manager", "the owner");
    assert_eq!(role_of(&page.items, contributor), "contributor", "default");
    assert_eq!(role_of(&page.items, reader), "reader");

    // A change is written; a user who is not a member changes nothing.
    assert!(db
        .fetch_scalar::<bool, _>(&SetMemberRoleQueryView::new(
            project,
            contributor,
            "manager"
        ))
        .await
        .unwrap());
    let stranger = create_user("Stranger", None).await;
    assert!(!db
        .fetch_scalar::<bool, _>(&SetMemberRoleQueryView::new(project, stranger, "reader"))
        .await
        .unwrap());
    // The owner keeps reading `manager` whatever their stored role.
    db.fetch_scalar::<bool, _>(&SetMemberRoleQueryView::new(project, owner, "reader"))
        .await
        .unwrap();
    let page: PagedRows<ProjectMemberRow> = db
        .fetch_one(&GetProjectUsersQueryView::new(project, 100, 0))
        .await
        .unwrap();
    assert_eq!(role_of(&page.items, contributor), "manager");
    assert_eq!(role_of(&page.items, owner), "manager");

    // The projects list carries the roles of its first members.
    let list = db
        .fetch_one::<ProjectsPage, _>(&GetProjectsQueryView::new(owner, 100, 0))
        .await
        .unwrap();
    let listed = list
        .items
        .iter()
        .find(|item| item.id() as u64 == project)
        .unwrap();
    assert_eq!(role_of(listed.members(), reader), "reader");
    assert_eq!(role_of(listed.members(), owner), "manager");

    assert!(db
        .fetch_scalar::<bool, _>(&IsProjectOwnerQueryView::new(project, owner))
        .await
        .unwrap());
    assert!(!db
        .fetch_scalar::<bool, _>(&IsProjectOwnerQueryView::new(project, reader))
        .await
        .unwrap());
}
