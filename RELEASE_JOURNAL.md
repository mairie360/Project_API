# Journal des releases — Project_API

<!-- release-journal-state {"dev": "bd75ffd463e66f39083e836154bc0b95f67d868d", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-04 18:27 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `bd75ffd` | 2026-11-04 18:27 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- L'API n'accepte plus les jetons HS256 : elle lit `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API) et n'a plus besoin de `JWT_SECRET`. Sans `JWT_SECRET`, tout jeton HS256 reçoit une 401. Les appelants (BFF, scripts) doivent présenter des jetons EdDSA signés par Core_API, et le déploiement doit fournir `JWT_PUBLIC_KEY`. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer` : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. Les erreurs de base sont décrites par SQLSTATE, contrainte, table et colonne au lieu du message Postgres. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Tests de matrice d'accès : `access-matrix.yaml` décrit l'accès à chaque opération de l'OpenAPI, et un test vérifie que l'API répond comme prévu (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)). Aucun changement de code métier.
- Image d'exécution passée en distroless Debian 13 et builder `rust:1.99-slim-trixie`, avec un test qui vérifie la version de Debian des images (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)).
- CI : CICD monté en v5.2.2 puis v5.3.1 (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210) ; [#214](https://github.com/mairie360/Project_API/pull/214)), checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), simulation RGPD (MAIR-497, [#214](https://github.com/mairie360/Project_API/pull/214)), et versions CICD alignées en v4.4.1 à v4.4.3 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).
- Mises à jour Renovate : actix-web 4.18.0, actix-http 3.20.0 (dev), log 0.4.35 (dev), tokio 1.54.1, et digests Docker épinglés (rust, redis, postgres, curl, nginx, zap, k6, newman, node).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- L'API n'accepte plus les jetons HS256 : elle lit `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API) et n'a plus besoin de `JWT_SECRET`. Sans `JWT_SECRET`, tout jeton HS256 reçoit une 401. Les appelants (BFF, scripts) doivent présenter des jetons EdDSA signés par Core_API, et le déploiement doit fournir `JWT_PUBLIC_KEY`. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer` : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. Les erreurs de base sont décrites par SQLSTATE, contrainte, table et colonne au lieu du message Postgres. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Tests de matrice d'accès : `access-matrix.yaml` décrit l'accès à chaque opération de l'OpenAPI, et un test vérifie que l'API répond comme prévu (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)). Aucun changement de code métier.
- Image d'exécution passée en distroless Debian 13 et builder `rust:1.99-slim-trixie`, avec un test qui vérifie la version de Debian des images (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)).
- CI : CICD monté en v5.2.2 puis v5.3.1 (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210) ; [#214](https://github.com/mairie360/Project_API/pull/214)), checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), simulation RGPD (MAIR-497, [#214](https://github.com/mairie360/Project_API/pull/214)), et versions CICD alignées en v4.4.1 à v4.4.3 (MAIR-612, [#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197)).
- Mises à jour Renovate : actix-web 4.18.0, actix-http 3.20.0 (dev), log 0.4.35 (dev), tokio 1.54.1, et digests Docker épinglés (rust, redis, postgres, curl, nginx, zap, k6, newman, node).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
