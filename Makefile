STAGE ?= "emission"

run:
	cargo run --no-default-features --features $(STAGE) main.c -p

test:
	cargo test --no-default-features --features $(STAGE)

tests-nora:
	./tests_ns/test_compiler ./target/debug/crome --chapter 13 --stage $(STAGE)
