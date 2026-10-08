pub mod access;
pub mod create;
pub mod delete;
pub mod get_project;
pub mod get_projects;
pub mod update;
pub mod update_status;

/// SQL condition "project `p` is visible to user `$1`", by role:
/// - Admin or Maire: every project;
/// - Responsable: their projects, those they are a member of, and those of their team (a member of
///   one of their groups owns it or is a member of it);
/// - other roles: the projects they own or are a member of.
///
/// The visible ids are collected once from the indexed lookups (owner, membership, team) instead
/// of evaluating correlated subqueries for every row of `projects`: on 5 000 projects, `GET
/// /projects/` went from 23 ms (agent) and 50 ms (Responsable) to 1-2 ms (MAIR-474).
///
/// Query views insert it with `concat!`: the user must be parameter `$1`.
#[macro_export]
macro_rules! project_visible_to_user_sql {
    () => {
        "(EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
            WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Maire')) \
         OR p.id IN ( \
            SELECT own.id FROM projects own WHERE own.owner_id = $1 \
            UNION ALL \
            SELECT pm.project_id FROM project_members pm WHERE pm.user_id = $1 \
            UNION ALL \
            SELECT team_project.id FROM group_members mine \
                JOIN group_members team ON team.group_id = mine.group_id \
                JOIN projects team_project ON team_project.owner_id = team.user_id \
            WHERE mine.user_id = $1 AND EXISTS (SELECT 1 FROM user_roles ur \
                JOIN roles r ON r.id = ur.role_id \
                WHERE ur.user_id = $1 AND r.name = 'Responsable') \
            UNION ALL \
            SELECT team_member.project_id FROM group_members mine \
                JOIN group_members team ON team.group_id = mine.group_id \
                JOIN project_members team_member ON team_member.user_id = team.user_id \
            WHERE mine.user_id = $1 AND EXISTS (SELECT 1 FROM user_roles ur \
                JOIN roles r ON r.id = ur.role_id \
                WHERE ur.user_id = $1 AND r.name = 'Responsable')))"
    };
}
