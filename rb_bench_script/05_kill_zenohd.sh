#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

read addr port < "$script_dir/config/router_addr.txt"

ssh -p "$port" "pi@$addr" \
    "sh -c 'pkill zenohd'"
