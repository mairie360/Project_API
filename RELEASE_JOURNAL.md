# Journal des releases — Project_API

<!-- release-journal-state {"dev": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "be2b41087e2072bcb9377f484d5037fdb9a969db", "updated": {"dev": "2026-10-21 01:49 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `be2b410` | 2026-10-20 17:07 UTC |
| dev | — | `5c1ed53` | 2026-10-21 01:49 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement
- Un push sur `main` ne lance plus les jobs Staging ni Prod : `cicd.yml` s'arrête à `release-dev`. La promotion se fait à la main avec `promote-staging.yml` ou `promote-prod.yml` (entrée optionnelle `sha`). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Corrections
- Le job `cargo audit` échouait à chaque run sur `main` (advisories publiées le 2026-10-20). Il passe désormais, grâce à `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 (RUSTSEC-2026-0381/0382/0383). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Pipeline et dépendances : `cicd.yml` passe de v4.2.2 à v4.4.0, le journal des releases reçoit ses secrets (`ANTHROPIC_API_KEY`, `RELEASE_APP_ID`, `RELEASE_APP_PK`), `utoipa-swagger-ui` passe en 10.0.2 et `tokio` en 1.54.0. (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181), [#183](https://github.com/mairie360/Project_API/pull/183), [#186](https://github.com/mairie360/Project_API/pull/186), [#189](https://github.com/mairie360/Project_API/pull/189))
- Tests et stacks compose : les tests d'intégration et les stacks `docker-compose*.yml` connectent l'API avec le rôle `project_api` au lieu du superuser, ce qui fait échouer en `permission denied` une requête hors des grants. Image de base `dev-949384c`, `mairie360_api_lib` 4.2.0. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Un push sur `main` ne lance plus les jobs Staging ni Prod : `cicd.yml` s'arrête à `release-dev`. La promotion se fait à la main avec `promote-staging.yml` ou `promote-prod.yml` (entrée optionnelle `sha`). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Corrections
- Le job `cargo audit` échouait à chaque run sur `main` (advisories publiées le 2026-10-20). Il passe désormais, grâce à `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 (RUSTSEC-2026-0381/0382/0383). (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Pipeline et dépendances : `cicd.yml` passe à mairie360/CICD v4.3.2 puis v4.4.0, et `utoipa-swagger-ui` passe en 10.0.2. ([#185](https://github.com/mairie360/Project_API/pull/185), [#186](https://github.com/mairie360/Project_API/pull/186), [#189](https://github.com/mairie360/Project_API/pull/189))
- Les workflows `promote-staging.yml` et `promote-prod.yml` sont ajoutés et épinglés sur la même version CICD que `cicd.yml`, avec `renovate.json` et la doc `CLAUDE.md` mis à jour. (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

## Différence staging → prod (en staging, pas encore en prod)

#### Technique
- Pipeline : `cicd.yml` passe de v4.2.2 à v4.3.2, et les secrets du journal des releases (`ANTHROPIC_API_KEY`, `RELEASE_APP_ID`, `RELEASE_APP_PK`) sont transmis au workflow CICD. (MAIR-571, [#178](https://github.com/mairie360/Project_API/pull/178), [#181](https://github.com/mairie360/Project_API/pull/181), [#182](https://github.com/mairie360/Project_API/pull/182), [#184](https://github.com/mairie360/Project_API/pull/184))
- Tests et stacks compose : les tests d'intégration et les stacks `docker-compose*.yml` connectent l'API avec le rôle `project_api` au lieu du superuser (`permission denied` sur les requêtes hors grants), `mairie360_api_lib` passe en 4.2.0 et `tokio` en 1.54.0. (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177), [#183](https://github.com/mairie360/Project_API/pull/183))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 01:49 UTC | dev | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39527971965) |
| 2026-10-20 18:24 UTC | dev | — | `56cb910` | [run](https://github.com/mairie360/Project_API/actions/runs/39472513794) |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
