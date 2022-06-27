#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir"

# while read -u10 rsize esize psize
# do
for rsize in 20 30 40 50 60 70 80 90 100
do
    for esize in 1 10 20 30 40 50 60 70 80 90
    do
        for psize in 128 256 512 1024 2048 4096 8192
        do
            if [ "$rsize" -le "$esize" ]
            then
                continue
            fi
            
            export rsize
            export esize
            export psize
            
            ts="$(date --rfc-3339=seconds | tr ' ' 'T')"
            name="${ts}_pi5x5_round-${rsize}_echo-${esize}_payload-${psize}"

            echo "Running experiment $name"

            ./run.sh

            # mkdir -p "$name"
            # mv exp_logs "$name"

#             sleep 10
        done
    done
done

# done 10<"$script_dir/config/params.txt"

popd
