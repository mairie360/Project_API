pub mod create;
pub mod delete;
pub mod get_project;
pub mod get_projects;
pub mod update;
pub mod update_status;

/// Condition SQL « le projet `p` est visible par l'utilisateur `$1` », selon son rôle :
/// - Admin ou Maire : tous les projets ;
/// - Responsable : ses projets, ceux dont il est membre, et ceux de son équipe (un membre d'un de
///   ses groupes en est propriétaire ou membre) ;
/// - autres rôles : les projets dont il est propriétaire ou membre.
///
/// Les vues de requête l'insèrent avec `concat!` : l'utilisateur doit être le paramètre `$1`.
#[macro_export]
macro_rules! project_visible_to_user_sql {
    () => {
        "(EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
            WHERE ur.user_id = $1 AND r.name IN ('Admin', 'Maire')) \
         OR p.owner_id = $1 \
         OR EXISTS (SELECT 1 FROM project_members pm \
            WHERE pm.project_id = p.id AND pm.user_id = $1) \
         OR (EXISTS (SELECT 1 FROM user_roles ur JOIN roles r ON r.id = ur.role_id \
                WHERE ur.user_id = $1 AND r.name = 'Responsable') \
             AND EXISTS (SELECT 1 FROM group_members mine \
                JOIN group_members team ON team.group_id = mine.group_id \
                WHERE mine.user_id = $1 \
                  AND (team.user_id = p.owner_id \
                    OR EXISTS (SELECT 1 FROM project_members team_member \
                        WHERE team_member.project_id = p.id \
                          AND team_member.user_id = team.user_id)))))"
    };
}
