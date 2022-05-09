#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

read addr port < "$script_dir/config/router_addr.txt"

log_file="/home/pi/log_exp_${log_time}.txt"
ssh -p "$port" "pi@$addr" \
    "bash -c 'env RUST_LOG=debug /home/pi/zenohd --no-timestamp > ${log_file} 2>&1'"
