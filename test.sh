#!/bin/bash
cargo build && ./gendata.py 1  | target/debug/scope

