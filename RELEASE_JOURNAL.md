# Journal des releases — Project_API

<!-- release-journal-state {"dev": "be2b41087e2072bcb9377f484d5037fdb9a969db", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "be2b41087e2072bcb9377f484d5037fdb9a969db", "updated": {"dev": "2026-10-19 20:22 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `be2b410` | 2026-10-20 17:07 UTC |
| dev | — | `be2b410` | 2026-10-19 20:22 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Les tests d'intégration et de endpoints, ainsi que les stacks `docker-compose*.yml`, se connectent désormais avec le rôle Postgres `project_api` comme en staging, au lieu du superuser. Une requête hors des grants de l'API fait donc échouer les tests (`permission denied`). Changements associés : nouveau test `tests/queries/db_role.rs`, `mairie360_api_lib` passé de 3.4.0 à 4.2.0 sans changement de code source, images `database` / `liquibase-migrations` en `dev-949384c`, avis `RUSTSEC-2026-0367/0368/0369` ignorés dans `.cargo/audit.toml`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- CI et dépendances, sans effet visible (4 PR) :
  - workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.3.1 ;
  - secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis pour le journal des releases ;
  - `tokio` mis à jour en 1.54.0.
  (MAIR-571, [#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#181](https://github.com/mairie360/Project_API/pull/181), [#183](https://github.com/mairie360/Project_API/pull/183))

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Technique
- Les tests d'intégration et de endpoints, ainsi que les stacks `docker-compose*.yml`, se connectent désormais avec le rôle Postgres `project_api` comme en staging, au lieu du superuser. Une requête hors des grants de l'API fait donc échouer les tests (`permission denied`). Changements associés : nouveau test `tests/queries/db_role.rs`, `mairie360_api_lib` passé de 3.4.0 à 4.2.0 sans changement de code source, images `database` / `liquibase-migrations` en `dev-949384c`, avis `RUSTSEC-2026-0367/0368/0369` ignorés dans `.cargo/audit.toml`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- CI et dépendances, sans effet visible (4 PR) :
  - workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.3.1 ;
  - secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis pour le journal des releases ;
  - `tokio` mis à jour en 1.54.0.
  (MAIR-571, [#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#181](https://github.com/mairie360/Project_API/pull/181), [#183](https://github.com/mairie360/Project_API/pull/183))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
