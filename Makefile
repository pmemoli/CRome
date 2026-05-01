run:
	cargo run main.c

test:
	cargo build
	./tests/test_compiler ./target/debug/crab --chapter 10 --stage parse
