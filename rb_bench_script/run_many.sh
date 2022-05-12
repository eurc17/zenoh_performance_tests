#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
pushd "$script_dir"

for rsize in 20 30 50 100
do
    for esize in 1 10 20 30 40 50
    do
        [ $(($esize < $rsize)) -eq 1 ] || continue
        
        for index in 1 2
        do
            ts="$(date --rfc-3339=seconds | tr ' ' 'T')"
            name="${ts}_round-${rsize}_echo-${esize}_$index"

            echo "Running experiment $name"

            echo "--total-put-number 1 \
--remote-pub-peers 11 \
--locators tcp/192.168.1.121:7447 \
--init-time 5000 \
--num-msgs-per-peer 30 \
--round-timeout 10000 \
--pub-interval 50 \
\
--peer-id PEER_ID \
--payload-size PAYLOAD_SIZE \
--output-dir OUTPUT_DIR \
\
--round-interval 50 \
--echo-interval 20 \
--max-rounds 3 \
--extra-rounds 1 \
--reliability reliable \
--sub-mode push \
--congestion-control drop" > config/command_args.template

            ./kill_remote.sh
            ./run.sh

            mkdir -p "$name"
            mv exp_logs "$name"
            
            ./kill_remote.sh
        done
    done
done

popd
