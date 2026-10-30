# Journal des releases — Project_API

<!-- release-journal-state {"dev": "59632677a529713c60589317738abd7c6c6d8fd8", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-10-21 21:32 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `5963267` | 2026-10-21 21:32 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Technique

- Pin CICD passé de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, en trois PR successives : v4.4.1 corrige une release prod qui ne se publiait pas, v4.4.2 corrige une erreur HTTP 403 à la création de la release, v4.4.3 pousse le tag avec le token de l'app release. Ces changements touchent le pipeline de release et n'ont pas d'effet sur l'API elle-même. (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197))
- Mise à jour de la dépendance `tokio` de 1.54.0 à 1.54.1 (patch) par Renovate. Le diff fourni est vide, le fichier modifié n'est donc pas vérifiable ici. ([#199](https://github.com/mairie360/Project_API/pull/199))
- Image de développement `rust:1.99-bookworm` épinglée sur le nouveau digest `5c1a173` dans `development.Dockerfile` uniquement. ([#195](https://github.com/mairie360/Project_API/pull/195))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique

- Pin CICD passé de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, en trois PR successives : v4.4.1 corrige une release prod qui ne se publiait pas, v4.4.2 corrige une erreur HTTP 403 à la création de la release, v4.4.3 pousse le tag avec le token de l'app release. Ces changements touchent le pipeline de release et n'ont pas d'effet sur l'API elle-même. (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197))
- Mise à jour de la dépendance `tokio` de 1.54.0 à 1.54.1 (patch) par Renovate. Le diff fourni est vide, le fichier modifié n'est donc pas vérifiable ici. ([#199](https://github.com/mairie360/Project_API/pull/199))
- Image de développement `rust:1.99-bookworm` épinglée sur le nouveau digest `5c1a173` dans `development.Dockerfile` uniquement. ([#195](https://github.com/mairie360/Project_API/pull/195))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 21:32 UTC | dev | — | `5963267` | [run](https://github.com/mairie360/Project_API/actions/runs/39677850753) |
| 2026-10-21 09:10 UTC | prod | v1.2.0 | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39573696361) |
| 2026-10-21 09:07 UTC | dev | — | `1a9a16d` | [run](https://github.com/mairie360/Project_API/actions/runs/39573548471) |
| 2026-10-21 08:27 UTC | dev | — | `8940e17` | [run](https://github.com/mairie360/Project_API/actions/runs/39567999372) |
| 2026-10-21 03:21 UTC | staging | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39537392264) |
| 2026-10-21 01:49 UTC | dev | — | `5c1ed53` | [run](https://github.com/mairie360/Project_API/actions/runs/39527971965) |
| 2026-10-20 18:24 UTC | dev | — | `56cb910` | [run](https://github.com/mairie360/Project_API/actions/runs/39472513794) |
| 2026-10-20 18:18 UTC | dev | — | `7877ea5` | [run](https://github.com/mairie360/Project_API/actions/runs/39472466012) |
| 2026-10-20 17:07 UTC | staging | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
| 2026-10-19 20:22 UTC | dev | — | `be2b410` | [run](https://github.com/mairie360/Project_API/actions/runs/39316639948) |
<!-- release-journal-history:end -->
