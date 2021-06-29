# Use latest stable rust toolchain
FROM rust:latest

# Switch working directory to 'app'
WORKDIR app

# Copy all the files
COPY . .

# Build the binary
RUN cargo build --release

# Set the entry point to our app
ENTRYPOINT [ "./target/release/zero2prod" ]
