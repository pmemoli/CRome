STAGE ?= "emission"

run:
	cargo build --no-default-features --features $(STAGE)
	cargo run --no-default-features main.c

test-crome:
	cargo test --no-default-features --features $(STAGE)

test-nora:
	./tests_ns/test_compiler ./target/debug/crome --chapter 13 --stage $(STAGE)

test:
	test-crome
	test-nora

run-debug:
	cargo build
	RUST_BACKTRACE=1 cargo run main.c -g

