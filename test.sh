#!/bin/sh

cargo build
./tests/test_compiler ./target/debug/crab --chapter 6
