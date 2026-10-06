# Same toolchain as the production `Dockerfile`, pinned by digest (Renovate bumps both together).
FROM rust:1.99-bookworm@sha256:fbc3a359627c6b5d9c8b20aae5c413a87392954f020006d7a9f7d95938964b23 AS development

RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch --locked

WORKDIR /usr/src/project

# --- DEPENDENCY CACHE ---
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# This layer stays cached as long as Cargo.toml and Cargo.lock do not change
RUN cargo build --locked && rm -rf src
# -----------------------------

COPY src ./src
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 3001
# Dev-only image (docker compose watch), never published: it needs root to write the bind-mounted target/ and cargo caches.
# nosemgrep: dockerfile.security.missing-user.missing-user
CMD ["/usr/local/bin/entrypoint.sh"]
