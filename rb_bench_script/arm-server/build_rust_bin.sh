#!/usr/bin/env bash
set -e  # exit when failing

if [ "$#" -ne 1 ] ; then
    echo "Usage: $0 BINARY_NAME"
    exit 1
fi

binary="$1"
script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

# Build
cargo build --release --bin "$binary"
