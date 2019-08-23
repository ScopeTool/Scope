#!/bin/bash
export RUST_BACKTRACE=1
cargo build && python2 ./gendata.py $1  | target/debug/scope

