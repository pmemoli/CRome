run:
	cargo run main.c

run-debug:
	RUST_BACKTRACE=1 cargo run main.c

test:
	cargo build
	./tests/test_compiler ./target/debug/crab --chapter 12 --stage parse
