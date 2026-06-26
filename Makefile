run:
	cargo build
	cargo run main.c -lm

run-debug:
	cargo build
	RUST_BACKTRACE=1 cargo run main.c -g

test:
	cargo build
	./tests/test_compiler ./target/debug/crab --chapter 13

