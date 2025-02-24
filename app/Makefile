PROJECT = yatch-app

run: build
	cd out && python -m http.server

build: out out/index.html
	EMCC_DEBUG=1 EMCC_CFLAGS="-O3 -sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sWASM=1 -sALLOW_MEMORY_GROWTH=1 -sWASM_MEM_MAX=512MB -sTOTAL_MEMORY=512MB -sABORTING_MALLOC=0 -sASYNCIFY -sFORCE_FILESYSTEM=1 -sASSERTIONS=1 -sERROR_ON_UNDEFINED_SYMBOLS=0 -sEXPORTED_RUNTIME_METHODS=ccallcwrap -lwebsocket.js" cargo build --release --target=wasm32-unknown-emscripten
	cp ../target/wasm32-unknown-emscripten/release/$(PROJECT).js ../target/wasm32-unknown-emscripten/release/$(subst -,_,$(PROJECT)).wasm out

out:
	mkdir out

out/index.html:
	cp index.html ./out/index.html

