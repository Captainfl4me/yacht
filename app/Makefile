PROJECT = yatch-app

run: build
	cd out && python -m http.server

build: out out/index.html out/gl.js
	cargo build --release --target=wasm32-unknown-unknown
	cp ../target/wasm32-unknown-unknown/release/$(PROJECT).* out

out:
	mkdir out

out/index.html: ./index.html
	cp index.html ./out/index.html

out/gl.js: ./assets/gl.js
	cp ./assets/gl.js ./out/gl.js 

