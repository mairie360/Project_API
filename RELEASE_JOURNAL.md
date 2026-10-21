# Journal des releases — Project_API

<!-- release-journal-state {"dev": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-10-21 01:49 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `5c1ed53` | 2026-10-21 01:49 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement

- Un push sur `main` ne livre plus que l'environnement dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se déclenchent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -R mairie360/Project_API [-f sha=SHA]`), avec l'approbation des environnements `Staging` et `Prod`. (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))
- Les stacks Docker Compose (`docker-compose.yml`, `docker-compose-integration.yml`, `-performance`, `-security`) connectent l'API avec le rôle Postgres `project_api` (mot de passe `project-api-password`) au lieu du superuser. Une requête hors des grants de Database échoue désormais avec `permission denied`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))

#### Technique

- `mairie360_api_lib` passe de 3.4.0 à 4.2.0, images `database` et `liquibase-migrations` et `TEST_DB_VERSION` en `dev-949384c`. Les tests d'intégration passent par le rôle `project_api`, les fixtures restent en superuser (`tests/common::fixtures_db()`), nouveau test `tests/queries/db_role.rs`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip 8, utoipa-swagger-ui, build script uniquement).
- `rustls-webpki` en 0.103.16 et `webpki-roots` / `webpki-root-certs` en 1.0.10 (RUSTSEC-2026-0381, 0382, 0383) : `cargo audit` ne bloque plus les runs sur main. ([#189](https://github.com/mairie360/Project_API/pull/189))
- Mises à jour de dépendances : tokio 1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)) et utoipa-swagger-ui 10.0.2 ([#186](https://github.com/mairie360/Project_API/pull/186)), diff non visible.
- `mairie360/CICD` passe de v4.2.1 à v4.4.0 en plusieurs PR ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#185](https://github.com/mairie360/Project_API/pull/185), [#189](https://github.com/mairie360/Project_API/pull/189)). Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow réutilisable pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Renovate regroupe les trois fichiers de workflow dans une même PR, et `CLAUDE.md` documente la promotion (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189)).

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement

- Un push sur `main` ne livre plus que l'environnement dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se déclenchent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -R mairie360/Project_API [-f sha=SHA]`), avec l'approbation des environnements `Staging` et `Prod`. (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))
- Les stacks Docker Compose (`docker-compose.yml`, `docker-compose-integration.yml`, `-performance`, `-security`) connectent l'API avec le rôle Postgres `project_api` (mot de passe `project-api-password`) au lieu du superuser. Une requête hors des grants de Database échoue désormais avec `permission denied`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))

#### Technique

- `mairie360_api_lib` passe de 3.4.0 à 4.2.0, images `database` et `liquibase-migrations` et `TEST_DB_VERSION` en `dev-949384c`. Les tests d'intégration passent par le rôle `project_api`, les fixtures restent en superuser (`tests/common::fixtures_db()`), nouveau test `tests/queries/db_role.rs`. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))
- `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip 8, utoipa-swagger-ui, build script uniquement).
- `rustls-webpki` en 0.103.16 et `webpki-roots` / `webpki-root-certs` en 1.0.10 (RUSTSEC-2026-0381, 0382, 0383) : `cargo audit` ne bloque plus les runs sur main. ([#189](https://github.com/mairie360/Project_API/pull/189))
- Mises à jour de dépendances : tokio 1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)) et utoipa-swagger-ui 10.0.2 ([#186](https://github.com/mairie360/Project_API/pull/186)), diff non visible.
- `mairie360/CICD` passe de v4.2.1 à v4.4.0 en plusieurs PR ([#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#185](https://github.com/mairie360/Project_API/pull/185), [#189](https://github.com/mairie360/Project_API/pull/189)). Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow réutilisable pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Renovate regroupe les trois fichiers de workflow dans une même PR, et `CLAUDE.md` documente la promotion (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 03:21 UTC | staging | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39537392264) |
| 2026-10-21 01:49 UTC | dev | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39527971965) |
| 2026-10-20 18:24 UTC | dev | — | `56cb910` | [run](https://github.com/mairie360/Project_API/actions/runs/39472513794) |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
