#!/usr/bin/env bash

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

if [ "$#" -ne 2 ] ; then
    echo "Usage: $0 LOCAL_FILE REMOTE_FILE"
    exit 1
fi

local_file="$1"
remote_file="$2"

while read addr port; do
    rsync -avz -e "ssh -p $port" "$local_file" "pi@$addr:$remote_file" &
done < "$script_dir/rpi_addrs.txt"

wait
