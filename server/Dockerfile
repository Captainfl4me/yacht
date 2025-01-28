
FROM rust:alpine AS builder
WORKDIR /usr/src/app
RUN apk add musl-dev
COPY . .
RUN cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/yatch-server /usr/local/bin/yatch-server
CMD ["yatch-server"]
