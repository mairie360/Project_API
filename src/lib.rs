// Ids cross the API as `u64` and the database as `INT4`: convert them with `id_to_sql` /
// `id_from_sql` (mairie360_api_lib), never with `as`, which wraps `2^32 + 1` to `1` (MAIR-422).
#![deny(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod database;
pub mod endpoints;
pub mod telemetry;
