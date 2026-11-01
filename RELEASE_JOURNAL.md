# Journal des releases — Project_API

<!-- release-journal-state {"dev": "26ec88b67857da787761289916105d2a2ee6a91e", "prod": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "prod_tag": "v1.2.0", "staging": "5c1ed53c25e52312242ebd2ce8fac9c5fd922ad3", "updated": {"dev": "2026-11-01 12:34 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `5c1ed53` | 2026-10-21 09:10 UTC |
| staging | — | `5c1ed53` | 2026-10-21 03:21 UTC |
| dev | — | `26ec88b` | 2026-11-01 12:34 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Corrections
- Journaux HTTP : la ligne de requête (query string) et le `Referer` ne sont plus journalisés. Avant : une valeur saisie dans une query string (nom, e-mail) pouvait se retrouver dans les journaux ; maintenant : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont écrits (d'après la description de la PR, la partie `src/request_log.rs` n'étant pas entièrement visible dans le diff). (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))
- Image d'exécution : avant, l'image `gcr.io/distroless/cc-debian12` embarquait une `libssl3` non corrigée (CVE-2026-84782) ; maintenant l'image est `gcr.io/distroless/cc-debian13:nonroot`, et le builder et `development.Dockerfile` passent aussi sur Debian 13 (trixie). (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))

#### Technique
- Matrice d'accès : `access-matrix.yaml` et `tests/endpoints/access_matrix.rs` vérifient que chaque opération de l'OpenAPI a une règle d'accès et que l'API répond selon elle ; ajout de `yaml_serde` en dev-dependency. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Test du marqueur RGPD : `gdpr-marker.yaml`, `docker-compose-gdpr-marker.yml` et `gdpr_marker_test.sh`, plus le workflow `gdpr-pr.yml` qui contrôle la section « Données personnelles » des PR, ignorant `RUSTSEC-2026-0387` dans `.cargo/audit.toml`. (MAIR-290 et MAIR-295, [#139](https://github.com/mairie360/Project_API/pull/139), [#143](https://github.com/mairie360/Project_API/pull/143))
- Dépendances : digests `rust:1.99-bookworm` de `development.Dockerfile` mis à jour trois fois (`5c1a173`, `48b7f43`, `9181b56`) ; `actix-web` 4.17.0 → 4.18.0 et `tokio` 1.54.0 → 1.54.1 (diffs non fournis, non vérifiés). (Renovate, [#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#203](https://github.com/mairie360/Project_API/pull/203), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199))
- Release MAIR-612 : version du workflow réutilisable mairie360/CICD passée de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, sans effet sur le code de l'API. ([#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197))

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Journaux HTTP : la ligne de requête (query string) et le `Referer` ne sont plus journalisés. Avant : une valeur saisie dans une query string (nom, e-mail) pouvait se retrouver dans les journaux ; maintenant : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont écrits (d'après la description de la PR, la partie `src/request_log.rs` n'étant pas entièrement visible dans le diff). (MAIR-290, [#139](https://github.com/mairie360/Project_API/pull/139))
- Image d'exécution : avant, l'image `gcr.io/distroless/cc-debian12` embarquait une `libssl3` non corrigée (CVE-2026-84782) ; maintenant l'image est `gcr.io/distroless/cc-debian13:nonroot`, et le builder et `development.Dockerfile` passent aussi sur Debian 13 (trixie). (MAIR-741, [#206](https://github.com/mairie360/Project_API/pull/206))

#### Technique
- Matrice d'accès : `access-matrix.yaml` et `tests/endpoints/access_matrix.rs` vérifient que chaque opération de l'OpenAPI a une règle d'accès et que l'API répond selon elle ; ajout de `yaml_serde` en dev-dependency. (MAIR-288, [#148](https://github.com/mairie360/Project_API/pull/148))
- Test du marqueur RGPD : `gdpr-marker.yaml`, `docker-compose-gdpr-marker.yml` et `gdpr_marker_test.sh`, plus le workflow `gdpr-pr.yml` qui contrôle la section « Données personnelles » des PR, ignorant `RUSTSEC-2026-0387` dans `.cargo/audit.toml`. (MAIR-290 et MAIR-295, [#139](https://github.com/mairie360/Project_API/pull/139), [#143](https://github.com/mairie360/Project_API/pull/143))
- Dépendances : digests `rust:1.99-bookworm` de `development.Dockerfile` mis à jour trois fois (`5c1a173`, `48b7f43`, `9181b56`) ; `actix-web` 4.17.0 → 4.18.0 et `tokio` 1.54.0 → 1.54.1 (diffs non fournis, non vérifiés). (Renovate, [#195](https://github.com/mairie360/Project_API/pull/195), [#200](https://github.com/mairie360/Project_API/pull/200), [#203](https://github.com/mairie360/Project_API/pull/203), [#201](https://github.com/mairie360/Project_API/pull/201), [#199](https://github.com/mairie360/Project_API/pull/199))
- Release MAIR-612 : version du workflow réutilisable mairie360/CICD passée de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, sans effet sur le code de l'API. ([#192](https://github.com/mairie360/Project_API/pull/192), [#194](https://github.com/mairie360/Project_API/pull/194), [#197](https://github.com/mairie360/Project_API/pull/197))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
