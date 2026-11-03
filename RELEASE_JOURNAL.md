# Journal des releases — Project_API

<!-- release-journal-state {"dev": "06b81b46b1d1c9c7536286bd02951934440f32ec", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-03 00:07 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `06b81b4` | 2026-11-03 00:07 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Corrections
- Les journaux de requêtes écrivaient la query string complète et le `Referer` (ex. `?limit=<nom>`). Maintenant ils ne gardent que l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée. Selon la description, les erreurs de la lib ne reprennent plus le message Postgres ni `DETAIL`, ni la valeur citée par serde. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- CI : les workflows passent de CICD v4.4.1 à v4.4.3 (promotion staging et prod, création du tag de release), puis à v5.2.2 avec les contrôles RGPD, et la checklist RGPD des PR est ajoutée. (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197), MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143), MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210))
- Image : runtime `cc-debian13:nonroot` et builder `rust:1.99-slim-trixie`, développement en trixie, pour corriger CVE-2026-84782. Aucune route ne change. (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))
- Matrice d'accès : ajout de `access-matrix.yaml` et de ses tests, sans changement de route. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Dépendances : digests `rust:1.99-bookworm` mis à jour (3 PR Renovate), actix-web 4.18.0 et tokio 1.54.1. Les diffs de ces deux dernières PR ne sont pas fournis. Ignore de RUSTSEC-2026-0387 dans `audit.toml`. ([#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199), [#203](https://github.com/mairie360/Project_API/pull/203))

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Les journaux de requêtes écrivaient la query string complète et le `Referer` (ex. `?limit=<nom>`). Maintenant ils ne gardent que l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée. Selon la description, les erreurs de la lib ne reprennent plus le message Postgres ni `DETAIL`, ni la valeur citée par serde. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- CI : les workflows passent de CICD v4.4.1 à v4.4.3 (promotion staging et prod, création du tag de release), puis à v5.2.2 avec les contrôles RGPD, et la checklist RGPD des PR est ajoutée. (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197), MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143), MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210))
- Image : runtime `cc-debian13:nonroot` et builder `rust:1.99-slim-trixie`, développement en trixie, pour corriger CVE-2026-84782. Aucune route ne change. (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))
- Matrice d'accès : ajout de `access-matrix.yaml` et de ses tests, sans changement de route. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Dépendances : digests `rust:1.99-bookworm` mis à jour (3 PR Renovate), actix-web 4.18.0 et tokio 1.54.1. Les diffs de ces deux dernières PR ne sont pas fournis. Ignore de RUSTSEC-2026-0387 dans `audit.toml`. ([#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199), [#203](https://github.com/mairie360/Project_API/pull/203))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
