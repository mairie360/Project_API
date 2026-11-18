//! Groups linked to a project: every current and future member of a linked group sees the
//! project, with the rights of a project member. A caller may link one of their own groups; Admin
//! and Maire may link any group.

use mairie360_api_lib::database::db_interface::{id_to_sql, ApiRequestDto, QueryParam};

/// SQL condition "group `g` may be linked by user `$1`": Admin and Maire may link any group, the
/// other roles only the groups they are a member of.
macro_rules! group_linkable_by_user_sql {
    () => {
        "(EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
            WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Maire')) \
         OR EXISTS (SELECT 1 FROM group_members gm WHERE gm.group_id = g.id AND gm.user_id = $1))"
    };
}

/// Whether user `user_id` may link group `group_id` to a project. `false` for an unknown group.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkableGroupQueryView {
    params: Vec<QueryParam>,
}

impl LinkableGroupQueryView {
    pub fn new(user_id: u64, group_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(group_id)),
            ],
        }
    }
}

impl ApiRequestDto for LinkableGroupQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT EXISTS (SELECT 1 FROM groups g WHERE g.id = $2 AND ",
            group_linkable_by_user_sql!(),
            ")"
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// A group a project can be linked to, as read by [`ListLinkableGroupsQueryView`] and in the
/// project detail.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GroupRow {
    pub id: i32,
    pub name: String,
}

/// The groups user `user_id` may link to a project, by name. One row: a JSON array of
/// [`GroupRow`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListLinkableGroupsQueryView {
    params: Vec<QueryParam>,
}

impl ListLinkableGroupsQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id))],
        }
    }
}

impl ApiRequestDto for ListLinkableGroupsQueryView {
    fn query_sql(&self) -> &'static str {
        concat!(
            "SELECT COALESCE(jsonb_agg(jsonb_build_object('id', g.id, 'name', g.name) \
                ORDER BY g.name, g.id), '[]'::jsonb) \
             FROM groups g WHERE ",
            group_linkable_by_user_sql!()
        )
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Makes `group_id` the only group linked to project `project_id`, or removes every link with
/// `None`. Linking the group already linked keeps its link date.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetProjectGroupQueryView {
    params: Vec<QueryParam>,
}

impl SetProjectGroupQueryView {
    pub fn new(project_id: u64, group_id: Option<u64>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(project_id)),
                QueryParam::OptionI32(group_id.map(id_to_sql)),
            ],
        }
    }
}

impl ApiRequestDto for SetProjectGroupQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH unlinked AS ( \
            DELETE FROM project_groups \
            WHERE project_id = $1 AND group_id IS DISTINCT FROM $2 \
         ) \
         INSERT INTO project_groups (project_id, group_id) \
         SELECT $1, $2 WHERE $2 IS NOT NULL \
         ON CONFLICT (project_id, group_id) DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }

    fn invalidated_read_caches(&self) -> &'static [&'static str] {
        crate::database::project::PROJECTS_READ_CACHES
    }
}
