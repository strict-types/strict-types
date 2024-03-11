#!/usr/bin/env bash

cargo run --features armor --bin std-stl -- --stl
cargo run --features armor --bin std-stl -- --sty
cargo run --features armor --bin std-stl -- --sta

cargo run --features armor --bin strict-stl -- --stl
cargo run --features armor --bin strict-stl -- --sty
cargo run --features armor --bin strict-stl -- --sta

cargo run --bin strict-vesper
