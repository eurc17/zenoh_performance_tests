#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e  # exit when failing

pushd "$repo_dir/reliable-broadcast-benchmark"
"$script_dir/build_rust_bin.sh" reliable-broadcast-benchmark
popd
