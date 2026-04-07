dev:
	RUST_BACKTRACE=1 cargo run main.c

test:
	cargo build
	./tests/test_compiler ./target/debug/crab --chapter 9 --stage parse
