# Journal des releases — Project_API

<!-- release-journal-state {"dev": "8940e1782f1aad6f92729a3995e4e6592fa6e2b1", "prod": "d52539be95623c7460881c1543f29176fab792ca", "prod_tag": "v1.1.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-10-21 08:27 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `d52539b` | — |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `8940e17` | 2026-10-21 08:27 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement
- Un push sur `main` ne livre plus staging ni prod : `cicd.yml` s'arrête à `release-dev`. Staging et prod se promeuvent à la main via `gh workflow run promote-staging.yml -R mairie360/Project_API` (et `promote-prod.yml`), avec `-f sha=<sha>` pour un commit donné ; les approbations des environnements `Staging` et `Prod` s'appliquent toujours. (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Dépendances : tokio 1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)), utoipa-swagger-ui 10.0.2 ([#186](https://github.com/mairie360/Project_API/pull/186)), mairie360_api_lib 4.2.0 (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177)), rustls-webpki 0.103.16 et webpki-roots / webpki-root-certs 1.0.10 pour débloquer `cargo audit` (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189)), image Rust de développement épinglée par digest ([#195](https://github.com/mairie360/Project_API/pull/195)).
- Pins mairie360/CICD de v4.2.2 à v4.4.1 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` ; les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis à `cicd.yml` pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Tests et stacks compose (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177)) : l'API s'y connecte avec le rôle `project_api`, les fixtures passent par le superuser, et les tests d'intégration échouent si une requête sort des grants de ce rôle.

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Dockerfile de développement : image `rust:1.99-bookworm` épinglée par digest `5c1a173` (Renovate, [#195](https://github.com/mairie360/Project_API/pull/195)) ; le Dockerfile de prod n'est pas dans ce diff.
- Pin mairie360/CICD v4.4.1 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, pour corriger la release prod qui n'avait pas abouti avec v4.4.0 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192)).

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne livre plus staging ni prod : `cicd.yml` s'arrête à `release-dev`. Staging et prod se promeuvent à la main via `gh workflow run promote-staging.yml -R mairie360/Project_API` (et `promote-prod.yml`), avec `-f sha=<sha>` pour un commit donné ; les approbations des environnements `Staging` et `Prod` s'appliquent toujours. (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189))

#### Technique
- Dépendances : tokio 1.54.0 ([#183](https://github.com/mairie360/Project_API/pull/183)), utoipa-swagger-ui 10.0.2 ([#186](https://github.com/mairie360/Project_API/pull/186)), mairie360_api_lib 4.2.0 (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177)), rustls-webpki 0.103.16 et webpki-roots / webpki-root-certs 1.0.10 pour débloquer `cargo audit` (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189)).
- Pins mairie360/CICD de v4.2.2 à v4.4.0 dans `cicd.yml`, avec les workflows `promote-staging.yml` et `promote-prod.yml` en v4.4.0 (MAIR-494, [#189](https://github.com/mairie360/Project_API/pull/189)), puis v4.4.1 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192)) ; les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis à `cicd.yml` pour le journal des releases (MAIR-571, [#181](https://github.com/mairie360/Project_API/pull/181)).
- Tests et stacks compose (MAIR-556, [#177](https://github.com/mairie360/Project_API/pull/177)) : l'API s'y connecte avec le rôle `project_api`, les fixtures passent par le superuser, et les tests d'intégration échouent si une requête sort des grants de ce rôle.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 08:27 UTC | dev | — | `8940e17` | [run](https://github.com/mairie360/Project_API/actions/runs/39567999372) |
| 2026-10-21 03:21 UTC | staging | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39537392264) |
| 2026-10-21 01:49 UTC | dev | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39527971965) |
| 2026-10-20 18:24 UTC | dev | — | `56cb910` | [run](https://github.com/mairie360/Project_API/actions/runs/39472513794) |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
