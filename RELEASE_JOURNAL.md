# Journal des releases — Project_API

<!-- release-journal-state {"dev": "9dc4965d6eff40989e34b7c44958a7a556d5ed48", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-04 01:51 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `9dc4965` | 2026-11-04 01:51 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les sessions ne sont plus signées avec le secret HS256 partagé mais avec la clé publique Ed25519 de Core_API : l'API lit `JWT_PUBLIC_KEY` (PEM) au lieu de `JWT_SECRET`. Tant que `JWT_SECRET` est défini, l'API accepte encore les HS256 signés avec lui (transition). Une fois ce secret retiré, tout token HS256 reçoit une 401 et l'API refuse de démarrer sans l'une des deux variables. Les appelants doivent présenter des jetons EdDSA émis par Core_API. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))

#### Corrections
- Les journaux de requêtes n'écrivent plus la query string ni l'en-tête Referer. Avant : un `?limit=<nom>` ou un `?offset=<e-mail>` refusé apparaissait dans les logs. Maintenant : seuls l'adresse client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Images et dépendances : images Docker épinglées par digest, Dockerfile d'exécution en distroless Debian 13 avec builder et image de dev sur rust trixie (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)), mises à jour actix-web 4.18.0 ([#201](https://github.com/mairie360/Project_API/pull/201)) et tokio 1.54.1 ([#199](https://github.com/mairie360/Project_API/pull/199)) dont le diff n'est pas fourni.
- CI et tests : CICD passé en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)), checklist RGPD des PR ([#143](https://github.com/mairie360/Project_API/pull/143)), matrice d'accès `access-matrix.yaml` sans changement de l'API (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)), stack de test du marqueur RGPD.

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les sessions ne sont plus signées avec le secret HS256 partagé mais avec la clé publique Ed25519 de Core_API : l'API lit `JWT_PUBLIC_KEY` (PEM) au lieu de `JWT_SECRET`. Tant que `JWT_SECRET` est défini, l'API accepte encore les HS256 signés avec lui (transition). Une fois ce secret retiré, tout token HS256 reçoit une 401 et l'API refuse de démarrer sans l'une des deux variables. Les appelants doivent présenter des jetons EdDSA émis par Core_API. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))

#### Corrections
- Les journaux de requêtes n'écrivent plus la query string ni l'en-tête Referer. Avant : un `?limit=<nom>` ou un `?offset=<e-mail>` refusé apparaissait dans les logs. Maintenant : seuls l'adresse client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique
- Images et dépendances : images Docker épinglées par digest, Dockerfile d'exécution en distroless Debian 13 avec builder et image de dev sur rust trixie (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)), mises à jour actix-web 4.18.0 ([#201](https://github.com/mairie360/Project_API/pull/201)) et tokio 1.54.1 ([#199](https://github.com/mairie360/Project_API/pull/199)) dont le diff n'est pas fourni.
- CI et tests : CICD passé en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)), checklist RGPD des PR ([#143](https://github.com/mairie360/Project_API/pull/143)), matrice d'accès `access-matrix.yaml` sans changement de l'API (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)), stack de test du marqueur RGPD.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
