FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build deps only
RUN cargo chef cook --release --recipe-path recipe.json
# All layers should stay cached unless deps change

# Build the app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod


FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - dynamically linked by some of our dependencies and therefore required.
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    && apt-get autoremove --purge -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]
