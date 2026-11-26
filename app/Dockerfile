FROM rust:bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked
WORKDIR /usr/src
COPY Cargo.toml Cargo.lock ./
COPY shared/ shared/
COPY server/ server/
COPY app/ app/
WORKDIR /usr/src/app
RUN trunk build --cargo-profile wasm-release

FROM nginx:alpine
COPY --from=builder /usr/src/app/dist/ /usr/share/nginx/html/
