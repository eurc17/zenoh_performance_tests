#!/usr/bin/env bash
set -e

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source "$script_dir/steps/00_config.sh"

# kill zenohd
read addr port < "$script_dir/config/router_addr.txt"
ssh -p "$port" "pi@$addr" "sh -c 'pkill zenohd || true'"

while read addr port peer_id name
do
    ssh -p "$port" "pi@$addr" "sh -c 'killall -KILL "$binary_name" >/dev/null 2>&1 || true'" < /dev/null \
        || echo "unable to connect to $addr:$port"
done < "$script_dir/config/rpi_addrs.txt"
