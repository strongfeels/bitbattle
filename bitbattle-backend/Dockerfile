# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache libgcc docker-cli

WORKDIR /app
COPY --from=builder /app/target/release/bitbattle-backend .

EXPOSE 4000

CMD ["./bitbattle-backend"]
