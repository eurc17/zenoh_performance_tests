#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir"

while read -u10 rsize esize psize
do
    export rsize
    export esize
    export psize
    
    ts="$(date --rfc-3339=seconds | tr ' ' 'T')"
    name="${ts}_round-${rsize}_echo-${esize}_payload-${psize}"

    echo "Running experiment $name"

    ./kill_remote.sh
    ./run.sh

    mkdir -p "$name"
    mv exp_logs "$name"
    ./kill_remote.sh

    sleep 10
done 10<"$script_dir/config/params.txt"

popd
