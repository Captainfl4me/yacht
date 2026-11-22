FROM rust:bookworm AS builder

WORKDIR /usr/src/app
RUN cargo install trunk --locked
COPY . .
RUN trunk build --cargo-profile wasm-release

FROM nginx:alpine
COPY --from=builder /usr/src/app/dist/ /usr/share/nginx/html/
