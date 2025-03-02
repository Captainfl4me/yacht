PROJECT = yatch-app

.PHONY: clean
.PHONY: ../target/wasm32-unknown-unknown/release/$(PROJECT).wasm

clean:
	rm -rf dist

run: build
	cd dist && python -m http.server

build: dist/index.html dist/$(PROJECT).js	
	cp ../target/wasm32-unknown-unknown/release/$(PROJECT).* dist

dist:
	mkdir dist

dist/index.html: dist ./index.html
	cp index.html ./dist/index.html

dist/$(PROJECT).js: dist ../target/wasm32-unknown-unknown/release/$(PROJECT).wasm
	wasm-bindgen ../target/wasm32-unknown-unknown/release/yatch-app.wasm --out-dir dist --target web --no-typescript
	sed -i "s/import \* as __wbg_star0 from 'env';//" dist/"$(PROJECT)".js
	sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" dist/"$(PROJECT)".js
	sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" dist/"$(PROJECT)".js
	sed -i "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" dist/"$(PROJECT)".js

../target/wasm32-unknown-unknown/release/$(PROJECT).wasm:
	cargo build --release --target=wasm32-unknown-unknown
