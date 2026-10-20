# Journal des releases — Project_API

<!-- release-journal-state {"dev": "7877ea5c1d2021a785bf06de9817adef3421cd84", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "be2b41087e2072bcb9377f484d5037fdb9a969db", "updated": {"dev": "2026-10-20 18:18 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `be2b410` | 2026-10-20 17:07 UTC |
| dev | — | `7877ea5` | 2026-10-20 18:18 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Tests d'intégration et stacks `docker-compose*.yml` : le code de l'API se connecte désormais à Postgres avec son propre rôle `project_api` au lieu du superuser, comme en staging et en prod. Les fixtures, Liquibase et le seeder restent en superuser, et le nouveau test `tests/queries/db_role.rs` vérifie les deux côtés. Les images `database` et `liquibase-migrations` passent en `dev-949384c` et `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans changement du code source. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- CI et dépendances : le workflow `mairie360/CICD` passe de v4.2.1 à v4.3.2 ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#185](https://github.com/mairie360/Project_API/pull/185)). Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)). `tokio` passe en v1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)).

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- CI : le workflow `mairie360/CICD` passe de v4.3.1 à v4.3.2. ([#185](https://github.com/mairie360/Project_API/pull/185))

## Différence staging → prod (en staging, pas encore en prod)

#### Technique
- Tests d'intégration et stacks `docker-compose*.yml` : le code de l'API se connecte désormais à Postgres avec son propre rôle `project_api` au lieu du superuser, comme en staging et en prod. Les fixtures, Liquibase et le seeder restent en superuser, et le nouveau test `tests/queries/db_role.rs` vérifie les deux côtés. Les images `database` et `liquibase-migrations` passent en `dev-949384c` et `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans changement du code source. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- CI et dépendances : le workflow `mairie360/CICD` passe de v4.2.1 à v4.3.1 ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184)). Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)). `tokio` passe en v1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
