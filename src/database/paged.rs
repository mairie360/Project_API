use serde::{Deserialize, Serialize};

/// One page of rows and the number of rows matching the query, as returned by a paginated query view.
///
/// Paginated views number their rows with `row_number() OVER (...) AS rn` and select
/// `paged_rows_sql!()` from them, so the total stays exact even when the page is past the end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedRows<T> {
    pub total: i64,
    pub items: Vec<T>,
}

/// `jsonb_build_object('total', …, 'items', […])` over a subquery aliased `t` whose rows carry
/// `rn`. The page bounds are the parameters given as `limit` and `offset` (e.g. `"$2"`, `"$3"`).
#[macro_export]
macro_rules! paged_rows_sql {
    ($limit:literal, $offset:literal) => {
        concat!(
            "SELECT jsonb_build_object('total', count(*), 'items', \
                COALESCE(jsonb_agg(to_jsonb(t) - 'rn' ORDER BY t.rn) \
                    FILTER (WHERE t.rn > ",
            $offset,
            "::bigint AND t.rn <= ",
            $offset,
            "::bigint + ",
            $limit,
            "::bigint), '[]'::jsonb))"
        )
    };
}
