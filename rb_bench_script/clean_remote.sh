#!/usr/bin/env bash
set -e

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source "$script_dir/steps/00_config.sh"

while read addr port peer_id name
do
    ssh -p "$port" "pi@$addr" "rm -rf $remote_dir" </dev/null
done < "$script_dir/config/rpi_addrs.txt"
