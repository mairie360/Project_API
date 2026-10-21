# Journal des releases — Project_API

<!-- release-journal-state {"dev": "1a9a16df2943d36f5b049791e5d58f2a96ad5fd1", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-10-21 09:07 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `1a9a16d` | 2026-10-21 09:07 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement
- Un push sur `main` ne déclenche plus la release de staging ni de prod : le workflow s'arrête à `release-dev`. Staging et prod se promeuvent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -f sha=<sha>`, approbation des environnements `Staging` et `Prod`). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Pins mairie360/CICD relevés au fil des PR, de v4.2.1 à v4.4.3, dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. Le correctif de `promote-prod` (création du tag et de la release) est arrivé avec v4.4.2 et v4.4.3. (MAIR-494, MAIR-571, MAIR-612, [#197](https://github.com/mairie360/Project_API/pull/197))
- Secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow CICD pour le journal de release de `main` (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Dépendances : `tokio` 1.54.0, `utoipa-swagger-ui` 10.0.2, `mairie360_api_lib` 3.4.0 vers 4.2.0 (sans changement de source selon la description de la PR ; le diff est tronqué), digest `rust:1.99-bookworm` dans `development.Dockerfile`, `rustls-webpki` et `webpki-roots` mis à jour, advisories RUSTSEC-2026-0367/0368/0369 ignorées dans `.cargo/audit.toml`. (MAIR-494, [#183](https://github.com/mairie360/Project_API/pull/183), [#186](https://github.com/mairie360/Project_API/pull/186), [#195](https://github.com/mairie360/Project_API/pull/195))
- Tests d'intégration et stacks compose : l'API s'y connecte avec le rôle `project_api` au lieu du superuser, images `database` et `liquibase-migrations` en `dev-949384c`, nouveau test `tests/queries/db_role.rs`. Le diff est tronqué, la lecture est donc partielle. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Pins mairie360/CICD passés de v4.4.1 à v4.4.2, puis v4.4.3, dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, pour corriger la création du tag et de la release par `promote-prod`. (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197))
- Digest `rust:1.99-bookworm` mis à jour dans `development.Dockerfile`, sans effet sur le service déployé. (MAIR-612, [#195](https://github.com/mairie360/Project_API/pull/195))

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne déclenche plus la release de staging ni de prod : le workflow s'arrête à `release-dev`. Staging et prod se promeuvent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -f sha=<sha>`, approbation des environnements `Staging` et `Prod`). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Pins mairie360/CICD relevés de v4.2.2 à v4.4.0 au fil des PR dans `cicd.yml` et les workflows de promotion. (MAIR-494, [#178](https://github.com/mairie360/Project_API/pull/178), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184), [#185](https://github.com/mairie360/Project_API/pull/185), [#189](https://github.com/mairie360/Project_API/pull/189))
- Secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow CICD pour le journal de release (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Dépendances : `tokio` 1.54.0, `utoipa-swagger-ui` 10.0.2, `mairie360_api_lib` 3.4.0 vers 4.2.0 (sans changement de source selon la description de la PR ; le diff est tronqué), `rustls-webpki` et `webpki-roots` mis à jour, advisories RUSTSEC-2026-0367/0368/0369 ignorées dans `.cargo/audit.toml`. (MAIR-494, MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177), [#183](https://github.com/mairie360/Project_API/pull/183), [#186](https://github.com/mairie360/Project_API/pull/186))
- Tests d'intégration et stacks compose : l'API s'y connecte avec le rôle `project_api` au lieu du superuser, images `database` et `liquibase-migrations` en `dev-949384c`. Le diff est tronqué, la lecture est donc partielle. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 09:07 UTC | dev | — | `1a9a16d` | [run](https://github.com/mairie360/Project_API/actions/runs/39573548471) |
| 2026-10-21 08:27 UTC | dev | — | `8940e17` | [run](https://github.com/mairie360/Project_API/actions/runs/39567999372) |
| 2026-10-21 03:21 UTC | staging | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39537392264) |
| 2026-10-21 01:49 UTC | dev | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39527971965) |
| 2026-10-20 18:24 UTC | dev | — | `56cb910` | [run](https://github.com/mairie360/Project_API/actions/runs/39472513794) |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
