#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

## The binary to be executed
binary_name="reliable-broadcast-benchmark"

## Timestamp used for logging
log_time="$(date +'%Y-%m-%d-%T')"

## Base directory on RPi
remote_dir="/home/pi/rb-exp"

## The directory to store program outputs on RPi
output_dir_suffix="test/payload/log_exp_$log_time"
remote_output_dir="$remote_dir/$output_dir_suffix"

## Zenoh repo and commit for compiling zenohd
zenoh_git_url="https://github.com/eclipse-zenoh/zenoh.git"
zenoh_git_commit="90539129b1a7c9e8c7d7daaa84138d093f71fedf"

## Space-delimited payload sizes
# payload_sizes=$(cat "$script_dir/../scripts/exp_payload_list.txt")
payload_sizes="128 256 512 1024 2048 4096 8192 16384"

## The RUST_LOG env set on RPi. It is intended for debug purpose.
remote_rust_log=""
# remote_rust_log="reliable_broadcast=debug,reliable_broadcast_benchmark=debug"
