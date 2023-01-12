
build: build-lib-dbg build-tests build-benches build-examples

build-lib:
	cargo build --release

build-lib-dbg:
	cargo build --debug


build-tests:
	cargo build --tests --debug

test:
	cargo test --debug


build-benches:
	cargo build --benches --release

bench:
	cargo bench --release


build-examples:
	cargo build --examples --release
