# Use the latest stable rust as the base image
FROM rust:1.68.2

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

# When the container is run, launch our binary
ENTRYPOINT [ "./target/release/zero2prod" ]
