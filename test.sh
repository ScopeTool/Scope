#!/bin/bash
export RUST_BACKTRACE=1
cargo build && ./gendata.py $1  | target/debug/scope

