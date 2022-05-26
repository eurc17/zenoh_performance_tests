#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e  # exit when failing

pushd "$repo_dir/reliable-broadcast-benchmark"
cargo build --release --bin "$binary"
popd
