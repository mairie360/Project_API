# Journal des releases — Project_API

<!-- release-journal-state {"dev": "80bfbe834f79cfd6670278420d35b6add09056ef", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-04 08:58 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `80bfbe8` | 2026-11-04 08:58 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les tokens de session sont désormais vérifiés avec `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM) au lieu de `JWT_SECRET`. Sans `JWT_PUBLIC_KEY`, la bibliothèque refuse de démarrer. Une fois `JWT_SECRET` retiré, tout token HS256 est refusé (401). Tant que `JWT_SECRET` est défini, les tokens HS256 signés avec lui restent acceptés pendant la transition. Les déploiements doivent donc poser `JWT_PUBLIC_KEY` avant de retirer `JWT_SECRET`, et les appelants doivent présenter des tokens EdDSA émis par Core_API. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))
- Les journaux de requêtes n’écrivent plus la query string ni le `Referer`. Ils gardent l’adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée. Les erreurs et journaux de `mairie360_api_lib` (4.2.0) ne reprennent plus les valeurs reçues : les erreurs base sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Image d’exécution distroless passée en `cc-debian13` avec un builder `rust:1.99-slim-trixie`, comme l’image de dev, et test `mair_741_every_image_runs_on_debian_13` (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)).
- Workflows CI et promotion montés en CICD v5.2.2, avec `ANTHROPIC_WORKSPACE_ID` transmis et test `mair_771_cicd_version` (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)). Les pins CICD précédents passaient de v4.4.1 à v4.4.3 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).
- Matrice d’accès `access-matrix.yaml` vérifiée contre l’OpenAPI par `tests/endpoints/access_matrix.rs`, sans changement de route (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)).
- Workflow de checklist RGPD sur les PR et test du marqueur RGPD (`gdpr-marker.yaml`, `gdpr_marker_test.sh`), plus une exception `RUSTSEC-2026-0387` dans `audit.toml` (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143) ; MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139)).
- Dépendances Rust mises à jour : actix-web 4.18.0 (MSRV 1.89), tokio 1.54.1, actix-http 3.20 et log 0.4.35 en dev-dependencies, `mairie360_api_lib` 4.3.0 (Renovate [#199](https://github.com/mairie360/Project_API/pull/199), [#201](https://github.com/mairie360/Project_API/pull/201), [#202](https://github.com/mairie360/Project_API/pull/202) ; MAIR-680).
- Images épinglées par digest : Rust (Renovate [#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#203](https://github.com/mairie360/Project_API/pull/203), [#216](https://github.com/mairie360/Project_API/pull/216)), et images des stacks compose (redis, postgres, nginx, node, curl, k6, newman, ZAP) par [#208](https://github.com/mairie360/Project_API/pull/208).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les tokens de session sont désormais vérifiés avec `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM) au lieu de `JWT_SECRET`. Sans `JWT_PUBLIC_KEY`, la bibliothèque refuse de démarrer. Une fois `JWT_SECRET` retiré, tout token HS256 est refusé (401). Tant que `JWT_SECRET` est défini, les tokens HS256 signés avec lui restent acceptés pendant la transition. Les déploiements doivent donc poser `JWT_PUBLIC_KEY` avant de retirer `JWT_SECRET`, et les appelants doivent présenter des tokens EdDSA émis par Core_API. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))
- Les journaux de requêtes n’écrivent plus la query string ni le `Referer`. Ils gardent l’adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée. Les erreurs et journaux de `mairie360_api_lib` (4.2.0) ne reprennent plus les valeurs reçues : les erreurs base sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Image d’exécution distroless passée en `cc-debian13` avec un builder `rust:1.99-slim-trixie`, comme l’image de dev, et test `mair_741_every_image_runs_on_debian_13` (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)).
- Workflows CI et promotion montés en CICD v5.2.2, avec `ANTHROPIC_WORKSPACE_ID` transmis et test `mair_771_cicd_version` (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)). Les pins CICD précédents passaient de v4.4.1 à v4.4.3 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).
- Matrice d’accès `access-matrix.yaml` vérifiée contre l’OpenAPI par `tests/endpoints/access_matrix.rs`, sans changement de route (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)).
- Workflow de checklist RGPD sur les PR et test du marqueur RGPD (`gdpr-marker.yaml`, `gdpr_marker_test.sh`), plus une exception `RUSTSEC-2026-0387` dans `audit.toml` (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143) ; MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139)).
- Dépendances Rust mises à jour : actix-web 4.18.0 (MSRV 1.89), tokio 1.54.1, actix-http 3.20 et log 0.4.35 en dev-dependencies, `mairie360_api_lib` 4.3.0 (Renovate [#199](https://github.com/mairie360/Project_API/pull/199), [#201](https://github.com/mairie360/Project_API/pull/201), [#202](https://github.com/mairie360/Project_API/pull/202) ; MAIR-680).
- Images épinglées par digest : Rust (Renovate [#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#203](https://github.com/mairie360/Project_API/pull/203), [#216](https://github.com/mairie360/Project_API/pull/216)), et images des stacks compose (redis, postgres, nginx, node, curl, k6, newman, ZAP) par [#208](https://github.com/mairie360/Project_API/pull/208).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-04 08:58 UTC | dev | — | `80bfbe8` | [run](https://github.com/mairie360/Project_API/actions/runs/41801599961) |
| 2026-11-04 01:51 UTC | dev | — | `9dc4965` | [run](https://github.com/mairie360/Project_API/actions/runs/41756068617) |
| 2026-11-03 01:37 UTC | dev | — | `4e63d9b` | [run](https://github.com/mairie360/Project_API/actions/runs/41559200343) |
| 2026-11-03 00:18 UTC | dev | — | `7979c87` | [run](https://github.com/mairie360/Project_API/actions/runs/41550219429) |
| 2026-11-03 00:07 UTC | dev | — | `06b81b4` | [run](https://github.com/mairie360/Project_API/actions/runs/41548452883) |
| 2026-11-01 12:34 UTC | dev | — | `26ec88b` | [run](https://github.com/mairie360/Project_API/actions/runs/41306153308) |
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
