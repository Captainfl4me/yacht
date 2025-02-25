PROJECT = yatch-app

run: build
	cd out && python -m http.server

build: out out/index.html
	EMCC_DEBUG=1 cargo build --release --target=wasm32-unknown-emscripten
	cp ../target/wasm32-unknown-emscripten/release/$(PROJECT).js ../target/wasm32-unknown-emscripten/release/$(subst -,_,$(PROJECT)).wasm out

out:
	mkdir out

out/index.html:
	cp index.html ./out/index.html

