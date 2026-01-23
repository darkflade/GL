FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY ./.sqlx ./.sqlx
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder app/target/release/GL ./GL
CMD ["./GL"]
