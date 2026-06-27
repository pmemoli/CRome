STAGE ?= "emission"

run:
	cargo build --no-default-features --features $(STAGE)
	cargo run --no-default-features main.c

test:
	# Crome tests
	cargo test --no-default-features --features $(STAGE)

	# Nora Sandler tests
	./tests_ns/test_compiler ./target/debug/crome --chapter 13 --stage $(STAGE)

test-crome:
	cargo test --no-default-features --features $(STAGE)

run-debug:
	cargo build
	RUST_BACKTRACE=1 cargo run main.c -g

