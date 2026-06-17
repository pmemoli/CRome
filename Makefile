run:
	cargo build
	cargo run main.c

run-debug:
	cargo build
	RUST_BACKTRACE=1 cargo run main.c

test:
	cargo build
	./tests/test_compiler ./target/debug/crab --chapter 13 --stage tacky

