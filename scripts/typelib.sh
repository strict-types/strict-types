#!/usr/bin/env bash

cargo run --features base64 --bin std-stl -- -s
cargo run --features base64 --bin std-stl -- -b
cargo run --features base64 --bin std-stl -- -h

cargo run --features base64 --bin strict-stl -- -s
cargo run --features base64 --bin strict-stl -- -b
cargo run --features base64 --bin strict-stl -- -h
