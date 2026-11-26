FROM rust:alpine AS builder
RUN apk add musl-dev
WORKDIR /usr/src
COPY server/ server/
COPY shared/ shared/
WORKDIR /usr/src/server
RUN cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/yatch-server /usr/local/bin/yatch-server
CMD ["yatch-server"]
