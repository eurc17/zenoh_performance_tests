#!/usr/bin/env bash
set -e

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
read addr port < "$script_dir/config/router_addr.txt"
ssh -p "$port" "pi@$addr" "sh -c 'pkill zenohd'"
