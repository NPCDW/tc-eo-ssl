FROM rust:latest AS rust-build

RUN mkdir /usr/src/tc-eo-ssl
WORKDIR /usr/src/tc-eo-ssl
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo build --release




FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y openssl ca-certificates
COPY --from=rust-build /usr/src/tc-eo-ssl/target/release/tc-eo-ssl /usr/local/bin/tc-eo-ssl
CMD tc-eo-ssl