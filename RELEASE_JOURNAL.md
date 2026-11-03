# Journal des releases — Project_API

<!-- release-journal-state {"dev": "7979c87f719be9aa10795fd086b7bbd8aa5d7c94", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-03 00:18 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `7979c87` | 2026-11-03 00:18 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Corrections
- Les journaux des requêtes n'affichent plus la query string ni l'en-tête `Referer` : avant, des valeurs saisies par les agents (par exemple `?limit=<nom>` ou `?offset=<e-mail>`) pouvaient y apparaître en clair ; maintenant seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))
- Les erreurs de base de données et de désérialisation de `mairie360_api_lib` 4.2.0 ne reprennent plus le message Postgres, `DETAIL` ni la valeur fournie : elles décrivent le SQLSTATE, la contrainte, la table et la colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Matrice d'accès `access-matrix.yaml` couvrant chaque opération de l'OpenAPI, avec un test qui vérifie les refus (401, 403, 404) par rôle et par relation, et ajout de la dev-dependance `yaml_serde` ; aucun changement de la logique d'accès dans les diffs fournis. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Image runtime passée en `gcr.io/distroless/cc-debian13`, builder et image de développement sur `rust:1.99-*-trixie`, avec un test `dockerfile_test` qui impose Debian 13. (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))
- Pins par digest des images de compose et Dockerfile, mises à jour Renovate des digests `rust:1.99` et de `actix-web` 4.18.0, `tokio` 1.54.1, exception d'audit `RUSTSEC-2026-0387` ; ces changements n'ont pas d'effet fonctionnel visible. ([#208](https://github.com/mairie360/Project_API/pull/208), [#203](https://github.com/mairie360/Project_API/pull/203), [#200](https://github.com/mairie360/Project_API/pull/200), [#195](https://github.com/mairie360/Project_API/pull/195), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199))
- Workflows CI : checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), marqueur de journaux RGPD (`gdpr-marker.yaml`, [#139](https://github.com/mairie360/Project_API/pull/139)), CICD passé de v4.4.0 à v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` ([#210](https://github.com/mairie360/Project_API/pull/210)) et montées de version de la promotion en release ([#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Les journaux des requêtes n'affichent plus la query string ni l'en-tête `Referer` : avant, des valeurs saisies par les agents (par exemple `?limit=<nom>` ou `?offset=<e-mail>`) pouvaient y apparaître en clair ; maintenant seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))
- Les erreurs de base de données et de désérialisation de `mairie360_api_lib` 4.2.0 ne reprennent plus le message Postgres, `DETAIL` ni la valeur fournie : elles décrivent le SQLSTATE, la contrainte, la table et la colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Matrice d'accès `access-matrix.yaml` couvrant chaque opération de l'OpenAPI, avec un test qui vérifie les refus (401, 403, 404) par rôle et par relation, et ajout de la dev-dependance `yaml_serde` ; aucun changement de la logique d'accès dans les diffs fournis. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Image runtime passée en `gcr.io/distroless/cc-debian13`, builder et image de développement sur `rust:1.99-*-trixie`, avec un test `dockerfile_test` qui impose Debian 13. (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))
- Pins par digest des images de compose et Dockerfile, mises à jour Renovate des digests `rust:1.99` et de `actix-web` 4.18.0, `tokio` 1.54.1, exception d'audit `RUSTSEC-2026-0387` ; ces changements n'ont pas d'effet fonctionnel visible. ([#208](https://github.com/mairie360/Project_API/pull/208), [#203](https://github.com/mairie360/Project_API/pull/203), [#200](https://github.com/mairie360/Project_API/pull/200), [#195](https://github.com/mairie360/Project_API/pull/195), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199))
- Workflows CI : checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), marqueur de journaux RGPD (`gdpr-marker.yaml`, [#139](https://github.com/mairie360/Project_API/pull/139)), CICD passé de v4.4.0 à v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` ([#210](https://github.com/mairie360/Project_API/pull/210)) et montées de version de la promotion en release ([#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
