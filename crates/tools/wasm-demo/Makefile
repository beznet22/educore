# educore-wasm-demo Makefile

# Builds the WASM module and regenerates the pkg/ directory.
# Requires: wasm-bindgen-cli 0.2.122, rustup target wasm32-unknown-unknown.

.PHONY: build clean serve test check

WASM_TARGET = wasm32-unknown-unknown
PKG_DIR = pkg
WASM_CRATE = educore-wasm-demo
BINDGEN_VERSION = 0.2.122

build:
	cargo build --target $(WASM_TARGET) --release -p $(WASM_CRATE)
	wasm-bindgen --target web --out-dir $(PKG_DIR) \
		target/$(WASM_TARGET)/release/$(WASM_CRATE).wasm

check:
	cargo check --target $(WASM_TARGET) -p $(WASM_CRATE)

test:
	cargo test -p $(WASM_CRATE)

serve: build
	python3 -m http.server 8080

clean:
	rm -rf $(PKG_DIR)
	cargo clean -p $(WASM_CRATE)

# Verify wasm-bindgen-cli version matches the project's wasm-bindgen dep.
verify-bindgen:
	@which wasm-bindgen > /dev/null || (echo "wasm-bindgen not installed; run: cargo install wasm-bindgen-cli --version $(BINDGEN_VERSION)" && exit 1)
	@echo "wasm-bindgen installed: $$(wasm-bindgen --version)"
