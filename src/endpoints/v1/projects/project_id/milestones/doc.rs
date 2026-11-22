use utoipa::OpenApi;

use crate::endpoints::v1::projects::project_id::milestones::endpoint::{
    __path_create_project_milestone, __path_delete_project_milestone,
    __path_get_project_milestones, __path_update_project_milestone,
};
use crate::endpoints::v1::projects::project_id::milestones::view::{
    CreateMilestoneView, GetProjectMilestonesResultView, MilestoneView, PatchMilestoneView,
};

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = Doc),
        (path = "/{milestone_id}", api = MilestoneIdDoc),
    )
)]
pub struct MilestonesDoc;

#[derive(OpenApi)]
#[openapi(
    paths(get_project_milestones, create_project_milestone),
    components(schemas(MilestoneView, GetProjectMilestonesResultView, CreateMilestoneView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest((path = "/", api = MilestoneIdOperationsDoc)))]
struct MilestoneIdDoc;

#[derive(OpenApi)]
#[openapi(
    paths(update_project_milestone, delete_project_milestone),
    components(schemas(PatchMilestoneView))
)]
struct MilestoneIdOperationsDoc;
