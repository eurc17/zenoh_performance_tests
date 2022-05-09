#!/usr/bin/env bash

if [ "$#" -ne 1 ] ; then
    echo "Usage: $0 BINARY_NAME"
    exit 1
fi

binary="$1"

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir/.."

"./target/release/$binary" $(cat "$script_dir/config/command_args")
