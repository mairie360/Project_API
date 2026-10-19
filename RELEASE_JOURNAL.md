# Journal des releases — Project_API

<!-- release-journal-state {"dev": "be2b41087e2072bcb9377f484d5037fdb9a969db", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "d52539be95623c7460881c1543f29176fab792ca", "updated": {"dev": "2026-10-19 20:22 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `d52539b` | — |
| dev | — | `be2b410` | 2026-10-19 20:22 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Les tests d'intégration et les stacks `docker-compose*.yml` se connectent désormais à Postgres avec le rôle `project_api` (mot de passe `project-api-password`, transmis à Liquibase via `-Dproject_api_password`) au lieu du superuser. Une requête non couverte par les grants de l'API échoue donc dès les tests. Le superuser reste réservé aux fixtures (`tests/common::fixtures_db()`), à postgres, à liquibase et au seeder. Changements associés : nouveau test `tests/queries/db_role.rs`, `mairie360_api_lib` passé de 3.4.0 à 4.2.0 sans modification du code source, images `database` / `liquibase-migrations` et `TEST_DB_VERSION` passées à `dev-949384c`, trois avis RUSTSEC de `zip` (dépendance de build de `utoipa-swagger-ui`) ignorés dans `.cargo/audit.toml`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184)) ; transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` à l'étape du journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)) ; mise à jour de `tokio` vers 1.54.0, dont le diff n'est pas fourni ([#183](https://github.com/mairie360/Project_API/pull/183)).

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Les tests d'intégration et les stacks `docker-compose*.yml` se connectent désormais à Postgres avec le rôle `project_api` (mot de passe `project-api-password`, transmis à Liquibase via `-Dproject_api_password`) au lieu du superuser. Une requête non couverte par les grants de l'API échoue donc dès les tests. Le superuser reste réservé aux fixtures (`tests/common::fixtures_db()`), à postgres, à liquibase et au seeder. Changements associés : nouveau test `tests/queries/db_role.rs`, `mairie360_api_lib` passé de 3.4.0 à 4.2.0 sans modification du code source, images `database` / `liquibase-migrations` et `TEST_DB_VERSION` passées à `dev-949384c`, trois avis RUSTSEC de `zip` (dépendance de build de `utoipa-swagger-ui`) ignorés dans `.cargo/audit.toml`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184)) ; transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` à l'étape du journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)) ; mise à jour de `tokio` vers 1.54.0, dont le diff n'est pas fourni ([#183](https://github.com/mairie360/Project_API/pull/183)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
