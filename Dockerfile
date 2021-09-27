FROM rust:1.53.0

WORKDIR /app

COPY . .

RUN cargo build --release

ENTRYPOINT [ "./target/release/zero2prod" ]
