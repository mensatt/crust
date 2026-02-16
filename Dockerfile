# syntax=docker/dockerfile:1

# Builder
FROM rust:1.93.1-alpine3.23 AS builder

RUN apk upgrade --no-cache && apk add --no-cache build-base perl
WORKDIR /usr/src/crust
COPY migrations ./migrations
COPY Cargo.lock Cargo.toml diesel.toml ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/crust/target \
	cargo build --locked --release && \
	cp target/release/crust /usr/src/crust/crust # Copy final binary to persistent path

# Runner
FROM alpine:3.23
COPY --from=builder /usr/src/crust/crust /usr/local/bin/crust
EXPOSE 8000
WORKDIR /
CMD ["crust"]
