#!/usr/bin/env bash

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

case $# in
    0)
        echo "Usage: $0 COMMAND [ARGS..]"
        exit 1
        ;;
    *)
        ;;
esac

while read addr port; do
    echo "# $addr:$port";
    ssh -p "$port" "pi@$addr" ${@:1} </dev/null
done < "$script_dir/rpi_addrs.txt"
