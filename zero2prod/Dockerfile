# Use the latest stable rust as the build image
FROM rust:1.68.2 AS builder

# Switch to the `/app` directory
WORKDIR /app

# Install the required system dependencies for our linking configuration (not rust default)
RUN apt-get update && apt-get install lld clang -y

# Copy all the files from the working environment into the Docker image
COPY . .

# Set sqlx to offline mode
ENV SQLX_OFFLINE true

# Build the binary using `release` profile for a nice, quick, optimized binary
RUN cargo build --release

# RUNTIME STAGE
FROM debian:bullseye-slim AS runtime

# Switch to the `/app` directory
WORKDIR /app

# Install OpenSSL - it's dynamically linked to our dependencies
# Install ca-certificates - needed to verify TLS certificates when establishing https connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy relevant files from the builder stage to this stage
COPY --from=builder /app/target/release/zero2prod zero2prod

# Get the configuration files for the runtime
COPY configuration configuration

# Set production mode
ENV APP_ENVIRONMENT production

# When the container is run, launch our binary
ENTRYPOINT [ "./target/release/zero2prod" ]
