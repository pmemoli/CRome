#!/bin/sh

cargo build
./tests/test_compiler ./target/debug/crab --chapter 9 --stage parse
