#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir"

exp_dt="$(date --rfc-3339=ns | tr ' ' 'T')"

# while read -u10 rsize esize psize
# do
for rsize in 10
do
    for esize in 0 2 4
    do
        for psize in 128 256 512 1024 2048 4096 8192
        do
            if [ "$rsize" -le "$esize" ]
            then
                continue
            fi

            exp_name="${exp_dt}_unlimited-zenoh_round-${rsize}_echo-${esize}_payload-${psize}"

            export rsize
            export esize
            export psize
            export exp_name

            ts="$(date --rfc-3339=seconds | tr ' ' 'T')"

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
