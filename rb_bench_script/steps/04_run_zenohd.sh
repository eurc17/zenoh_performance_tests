#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

read addr port < "$script_dir/config/router_addr.txt"

log_file="~/rb_exp/zenohd_${log_time}.txt"

ssh -p "$port" "pi@$addr" "mkdir -p ~/rb_exp"
rsync -aPz -e "ssh -p $port" \
      "$script_dir/files/zenohd" \
      "pi@$addr:~/rb_exp/zenohd"
ssh -p "$port" "pi@$addr" "pkill zenohd || true"
ssh -p "$port" "pi@$addr" \
    "bash -c 'env RUST_LOG=debug ~/rb_exp/zenohd --no-timestamp > ${log_file} 2>&1 &'"

# make sure zenohd is ready
sleep 5
