use utoipa::OpenApi;

use crate::endpoints::v1::project_templates::endpoint::{
    __path_create_template, __path_delete_template, __path_get_template, __path_get_templates,
    __path_update_template,
};
use crate::endpoints::v1::project_templates::view::{
    CreateTemplateResultView, CreateTemplateView, GetTemplatesResultView, PatchTemplateView,
    ProjectTemplateView, TemplateStatusInputView, TemplateStatusView,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_templates,
        create_template,
        get_template,
        update_template,
        delete_template
    ),
    components(schemas(
        ProjectTemplateView,
        TemplateStatusView,
        GetTemplatesResultView,
        TemplateStatusInputView,
        CreateTemplateView,
        CreateTemplateResultView,
        PatchTemplateView
    ))
)]
pub struct TemplatesDoc;
