# Journal des releases — Project_API

<!-- release-journal-state {"dev": "4e63d9b498f0ac8c16f9c12845bb9bca558be205", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-03 01:37 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `4e63d9b` | 2026-11-03 01:37 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement

- Jetons de session : l'API ne valide plus les sessions avec `JWT_SECRET`, mais avec `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM). Les jetons EdDSA signés par Core_API sont acceptés. Sans `JWT_SECRET` défini, tout jeton HS256 est refusé avec `401`. Tant que `JWT_SECRET` est défini, la lib accepte encore les HS256 signés avec lui (transition). Un déploiement doit donc fournir `JWT_PUBLIC_KEY` puis retirer `JWT_SECRET`. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))

#### Corrections

- Journaux des requêtes : avant, la ligne de requête (query string) et le `Referer` étaient journalisés, donc une valeur refusée comme `?limit=<nom>` ou `?offset=<e-mail>` apparaissait dans les logs. Maintenant seuls l'adresse client, la méthode, le chemin sans query string, le statut, la taille et la durée sont journalisés. Les erreurs de la lib ne reprennent plus la valeur reçue : les erreurs base de données sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique

- Dépendances et images : `mairie360_api_lib` passe de 4.2.0 à 4.3.0, actix-web à 4.18.0 et tokio à 1.54.1 (sans diff visible). Les images de Docker Compose et des Dockerfile sont épinglées par digest (redis, postgres, nginx, node, curl, k6, newman, ZAP, rust 1.99). Le runtime distroless passe de Debian 12 à Debian 13, avec un builder trixie (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)). Une exception `RUSTSEC-2026-0387` est ajoutée à `audit.toml` (MAIR-290).
- CI et tests : CICD passe de v4.4.1 à v4.4.2, puis v4.4.3, puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-612, MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)). Ajout de la checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), de la matrice d'accès `access-matrix.yaml` et de ses tests (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)), et du test du marqueur RGPD dans les logs (MAIR-290). Les scripts de test signent désormais les jetons EdDSA (MAIR-680).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement

- Jetons de session : l'API ne valide plus les sessions avec `JWT_SECRET`, mais avec `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM). Les jetons EdDSA signés par Core_API sont acceptés. Sans `JWT_SECRET` défini, tout jeton HS256 est refusé avec `401`. Tant que `JWT_SECRET` est défini, la lib accepte encore les HS256 signés avec lui (transition). Un déploiement doit donc fournir `JWT_PUBLIC_KEY` puis retirer `JWT_SECRET`. (MAIR-680, [#212](https://github.com/mairie360/Project_API/pull/212))

#### Corrections

- Journaux des requêtes : avant, la ligne de requête (query string) et le `Referer` étaient journalisés, donc une valeur refusée comme `?limit=<nom>` ou `?offset=<e-mail>` apparaissait dans les logs. Maintenant seuls l'adresse client, la méthode, le chemin sans query string, le statut, la taille et la durée sont journalisés. Les erreurs de la lib ne reprennent plus la valeur reçue : les erreurs base de données sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))

#### Technique

- Dépendances et images : `mairie360_api_lib` passe de 4.2.0 à 4.3.0, actix-web à 4.18.0 et tokio à 1.54.1 (sans diff visible). Les images de Docker Compose et des Dockerfile sont épinglées par digest (redis, postgres, nginx, node, curl, k6, newman, ZAP, rust 1.99). Le runtime distroless passe de Debian 12 à Debian 13, avec un builder trixie (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206)). Une exception `RUSTSEC-2026-0387` est ajoutée à `audit.toml` (MAIR-290).
- CI et tests : CICD passe de v4.4.1 à v4.4.2, puis v4.4.3, puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-612, MAIR-771, [#210](https://github.com/mairie360/Project_API/pull/210)). Ajout de la checklist RGPD sur les PR (MAIR-295, [#143](https://github.com/mairie360/Project_API/pull/143)), de la matrice d'accès `access-matrix.yaml` et de ses tests (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148)), et du test du marqueur RGPD dans les logs (MAIR-290). Les scripts de test signent désormais les jetons EdDSA (MAIR-680).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
