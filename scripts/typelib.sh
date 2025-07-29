#!/usr/bin/env bash

cargo run --features stl --bin std-stl -- --stl
cargo run --features stl --bin std-stl -- --sty
cargo run --features stl --bin std-stl -- --sta

cargo run --features stl --bin strict-stl -- --stl
cargo run --features stl --bin strict-stl -- --sty
cargo run --features stl --bin strict-stl -- --sta

cargo run --features stl --bin strict-vesper
