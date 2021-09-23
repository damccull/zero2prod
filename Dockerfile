# FROM lukemathwalker/cargo-chef as planner
# WORKDIR /app
# COPY . .
# # Create a lock-like file for the project
# RUN cargo chef prepare --recipe-path recipe.json

# FROM lukemathwalker/cargo-chef as cacher
# WORKDIR /app
# COPY --from=planner /app/recipe.json recipe.json
# # Build the project dependencies, not the app
# RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:latest as cacher
WORKDIR /app
# Build the project dependencies, not the app
COPY Cargo.toml Cargo.lock ./
RUN cargo install cargo-build-deps
RUN cargo build-deps --release

FROM rust:latest as builder
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV SQLX_OFFLINE true
#Build the application in release mode, leveraging the cached dependencies
RUN cargo build --release --bin zero2prod

FROM debian:buster-slim as runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked to some of the dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # clean up
    && apt-get autoremove --purge -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]
