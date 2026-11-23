pub mod access;
pub mod create;
pub mod delete;
pub mod get_project;
pub mod get_projects;
pub mod group;
pub mod update;
pub mod update_status;

use mairie360_api_lib::smart_db::ReadCache;

/// Read cache of `GET /api/v1/projects/` (MAIR-524): one entry per caller and query string, kept
/// 30 seconds. Every write view that changes what the list shows (a project, its tasks, its
/// members) lists [`PROJECTS_READ_CACHES`] in `invalidated_read_caches`, so a write through this
/// API is visible on the next read. Roles and groups live in `Core_API`, which cannot invalidate
/// it: a visibility change made there shows within the TTL.
pub const PROJECTS_READ_CACHE: ReadCache = ReadCache::new("projects", 30);

/// The read caches a write on a project, a task or a membership invalidates.
pub const PROJECTS_READ_CACHES: &[&str] = &[PROJECTS_READ_CACHE.namespace()];

/// SQL condition "project `p` is visible to user `$1`", by role:
/// - Admin or Maire: every project;
/// - Responsable: their projects, those they are a member of, and those of their team (a member of
///   one of their groups owns it or is a member of it);
/// - other roles: the projects they own or are a member of.
///
/// Whatever the role, a project linked to one of the user's groups (`project_groups`) is visible.
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
                WHERE ur.user_id = $1 AND r.name = 'Responsable') \
            UNION ALL \
            SELECT linked.project_id FROM group_members mine \
                JOIN project_groups linked ON linked.group_id = mine.group_id \
            WHERE mine.user_id = $1))"
    };
}

/// SQL condition "user `$1` manages project `p`" (MAIR-1088): an Admin, a Maire or a Responsable
/// (who must also see the project, checked apart), its owner, or a member whose project role is
/// `manager`. Query views insert it with `concat!`: the user must be parameter `$1`, the project
/// row `p`.
#[macro_export]
macro_rules! manages_project_sql {
    () => {
        "(EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
            WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Maire', 'Responsable')) \
          OR p.owner_id = $1 \
          OR EXISTS (SELECT 1 FROM project_members mm WHERE mm.project_id = p.id \
            AND mm.user_id = $1 AND mm.role = 'manager'))"
    };
}

/// SQL of the role of user `$1` in project `p` (MAIR-1088): `manager` for its owner, the
/// `project_members.role` of a member, `NULL` otherwise.
#[macro_export]
macro_rules! caller_project_role_sql {
    () => {
        "(CASE WHEN p.owner_id = $1 THEN 'manager' ELSE (SELECT mr.role::text \
            FROM project_members mr WHERE mr.project_id = p.id AND mr.user_id = $1) END)"
    };
}
