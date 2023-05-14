#!/usr/bin/env bash

cargo run --features base64 --bin std-stl -- --stl
cargo run --features base64 --bin std-stl -- --sty
cargo run --features base64 --bin std-stl -- --sta

cargo run --features base64 --bin strict-stl -- --stl
cargo run --features base64 --bin strict-stl -- --sty
cargo run --features base64 --bin strict-stl -- --sta
