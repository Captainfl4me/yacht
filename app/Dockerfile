FROM emscripten/emsdk:latest AS builder

WORKDIR /usr/src/app

RUN apt update && apt upgrade -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable && rustup target add wasm32-unknown-emscripten
RUN apt-get install cmake clang -y

COPY . .

RUN EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY -sASSERTIONS" cargo build --release --target=wasm32-unknown-emscripten

FROM nginx:alpine AS run

COPY ./index.html /usr/share/nginx/html/
COPY --from=builder /usr/src/app/target/wasm32-unknown-emscripten/release/yatch-app.* /usr/share/nginx/html/
COPY --from=builder /usr/src/app/target/wasm32-unknown-emscripten/release/yatch_app.* /usr/share/nginx/html/
