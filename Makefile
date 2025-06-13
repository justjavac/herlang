.PHONY: setup
setup:
	(cd web && yarn)

.PHONY: start
start:
	make build_wasm
	(cd web && yarn start)

.PHONY: test
test:
	cargo test

.PHONY: build_repl
build_repl:
	cargo build --release --features="binaries"

build_wasm: web/src/herlang.wasm

web/src/herlang.wasm: target/wasm32-unknown-unknown/tiny/wasm.wasm
	if command -v wasm-opt >/dev/null; then \
		wasm-opt --strip-debug -Oz -o web/src/herlang.wasm target/wasm32-unknown-unknown/tiny/wasm.wasm; \
	else \
		printf '⚠  %s\n' "wasm-opt (binaryen) not found: binary will be larger than normal." >&2; \
	    cp target/wasm32-unknown-unknown/tiny/wasm.wasm web/src/herlang.wasm; \
	fi

target/wasm32-unknown-unknown/tiny/wasm.wasm: FORCE
	cargo build --bin wasm -Z unstable-options --profile tiny --target wasm32-unknown-unknown --features=wasm

FORCE:

.PHONY: web_deploy
web_deploy:
	make build_wasm
	(cd web && yarn --pure-lockfile && yarn deploy)

.PHONY: repl
repl:
	cargo run --bin herlang --features="binaries"
