# Journal des releases — Project_API

<!-- release-journal-state {"dev": "9a2f2514d59dd69f6c98a9ecb13cbd048df21c4f", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "9a2f2514d59dd69f6c98a9ecb13cbd048df21c4f", "updated": {"dev": "2026-11-12 12:57 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-11-12 18:51 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `9a2f251` | 2026-11-12 18:51 UTC |
| dev | — | `9a2f251` | 2026-11-12 12:57 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

- fix: MAIR-262 Integration tests - API functional tests, BFF business tests and regression tests (#228) ([#228](https://github.com/mairie360/Project_API/pull/228))
- chore(deps): update gcr.io/distroless/cc-debian13:nonroot docker digest to cc66d22 (#222) ([#222](https://github.com/mairie360/Project_API/pull/222))
- fix(deps): MAIR-740 APIs: fix the cargo audit RustSec alerts (actix-http, libsqlite3-sys) (#227) ([#227](https://github.com/mairie360/Project_API/pull/227))
- chore(deps): update rust:1.99-trixie docker digest to 8a4e90c (#221) ([#221](https://github.com/mairie360/Project_API/pull/221))
- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#220) ([#220](https://github.com/mairie360/Project_API/pull/220))
- chore(deps): pin node.js to c86ecea (#217) ([#217](https://github.com/mairie360/Project_API/pull/217))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#214](https://github.com/mairie360/Project_API/pull/214))
- chore(deps): update rust dependencies (#202) ([#202](https://github.com/mairie360/Project_API/pull/202))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#216) ([#216](https://github.com/mairie360/Project_API/pull/216))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#212](https://github.com/mairie360/Project_API/pull/212))
- chore(deps): pin dependencies (#208) ([#208](https://github.com/mairie360/Project_API/pull/208))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#210) ([#210](https://github.com/mairie360/Project_API/pull/210))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#206) ([#206](https://github.com/mairie360/Project_API/pull/206))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#143) ([#143](https://github.com/mairie360/Project_API/pull/143))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#148) ([#148](https://github.com/mairie360/Project_API/pull/148))
- chore(deps): update rust:1.99-bookworm docker digest to 9181b56 (#203) ([#203](https://github.com/mairie360/Project_API/pull/203))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#139) ([#139](https://github.com/mairie360/Project_API/pull/139))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#200) ([#200](https://github.com/mairie360/Project_API/pull/200))
- chore(deps): update rust crate actix-web to v4.18.0 (#201) ([#201](https://github.com/mairie360/Project_API/pull/201))
- chore(deps): update rust crate tokio to v1.54.1 (#199) ([#199](https://github.com/mairie360/Project_API/pull/199))
- ci: MAIR-612 Release prod: align internal versions and deploy (#197) ([#197](https://github.com/mairie360/Project_API/pull/197))
- ci: MAIR-612 Release prod: align internal versions and deploy (#194) ([#194](https://github.com/mairie360/Project_API/pull/194))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#195) ([#195](https://github.com/mairie360/Project_API/pull/195))
- ci: MAIR-612 Release prod: align internal versions and deploy (#192) ([#192](https://github.com/mairie360/Project_API/pull/192))

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

- fix: MAIR-262 Integration tests - API functional tests, BFF business tests and regression tests (#228) ([#228](https://github.com/mairie360/Project_API/pull/228))
- chore(deps): update gcr.io/distroless/cc-debian13:nonroot docker digest to cc66d22 (#222) ([#222](https://github.com/mairie360/Project_API/pull/222))
- fix(deps): MAIR-740 APIs: fix the cargo audit RustSec alerts (actix-http, libsqlite3-sys) (#227) ([#227](https://github.com/mairie360/Project_API/pull/227))
- chore(deps): update rust:1.99-trixie docker digest to 8a4e90c (#221) ([#221](https://github.com/mairie360/Project_API/pull/221))
- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#220) ([#220](https://github.com/mairie360/Project_API/pull/220))
- chore(deps): pin node.js to c86ecea (#217) ([#217](https://github.com/mairie360/Project_API/pull/217))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#214](https://github.com/mairie360/Project_API/pull/214))
- chore(deps): update rust dependencies (#202) ([#202](https://github.com/mairie360/Project_API/pull/202))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#216) ([#216](https://github.com/mairie360/Project_API/pull/216))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#212](https://github.com/mairie360/Project_API/pull/212))
- chore(deps): pin dependencies (#208) ([#208](https://github.com/mairie360/Project_API/pull/208))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#210) ([#210](https://github.com/mairie360/Project_API/pull/210))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#206) ([#206](https://github.com/mairie360/Project_API/pull/206))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#143) ([#143](https://github.com/mairie360/Project_API/pull/143))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#148) ([#148](https://github.com/mairie360/Project_API/pull/148))
- chore(deps): update rust:1.99-bookworm docker digest to 9181b56 (#203) ([#203](https://github.com/mairie360/Project_API/pull/203))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#139) ([#139](https://github.com/mairie360/Project_API/pull/139))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#200) ([#200](https://github.com/mairie360/Project_API/pull/200))
- chore(deps): update rust crate actix-web to v4.18.0 (#201) ([#201](https://github.com/mairie360/Project_API/pull/201))
- chore(deps): update rust crate tokio to v1.54.1 (#199) ([#199](https://github.com/mairie360/Project_API/pull/199))
- ci: MAIR-612 Release prod: align internal versions and deploy (#197) ([#197](https://github.com/mairie360/Project_API/pull/197))
- ci: MAIR-612 Release prod: align internal versions and deploy (#194) ([#194](https://github.com/mairie360/Project_API/pull/194))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#195) ([#195](https://github.com/mairie360/Project_API/pull/195))
- ci: MAIR-612 Release prod: align internal versions and deploy (#192) ([#192](https://github.com/mairie360/Project_API/pull/192))

> Résumé Claude indisponible pour ce passage : listes brutes des commits.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-12 18:51 UTC | staging | — | `9a2f251` | [run](https://github.com/mairie360/Project_API/actions/runs/43282695272) |
| 2026-11-12 12:57 UTC | dev | — | `9a2f251` | [run](https://github.com/mairie360/Project_API/actions/runs/43220653995) |
| 2026-11-12 11:53 UTC | dev | — | `5de1a25` | [run](https://github.com/mairie360/Project_API/actions/runs/43211595347) |
| 2026-11-06 05:18 UTC | dev | — | `caf3327` | [run](https://github.com/mairie360/Project_API/actions/runs/42166743871) |
| 2026-11-04 18:27 UTC | dev | — | `bd75ffd` | [run](https://github.com/mairie360/Project_API/actions/runs/41883079528) |
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
