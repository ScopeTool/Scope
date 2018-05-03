#!/bin/bash
cargo build && ./gendata.py | target/debug/scope

