#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

sleep_for='3 seconds'

echo "Running test for round_interval=$rsize echo_interval=$esize payload_size=$psize"

start_time="$(date --date="$sleep_for" +%s)"
log_dir="$script_dir/logs/rb_${log_time}_${psize}_${rsize}_${esize}"
peer_id=0

while read peer_name
do
    exp_name="256kbps-zenoh_round-${rsize}_echo-${esize}_payload-${psize}"
    output_dir="${script_dir}/${exp_name}/exp_logs/${peer_name}/log_exp_${log_time}/test/payload/log_exp_${log_time}"
    mkdir -p "$output_dir"

    # the binary program
    program="$repo_dir/target/release/$binary_name"

    # the command options derived from command_args.template
    export PAYLOAD_SIZE=${psize}
    export ROUND_INTERVAL=${rsize}
    export ECHO_INTERVAL=${esize}
    export OUTPUT_DIR="${output_dir}"
    export PEER_ID=${peer_id}
    export RUST_LOG=${remote_rust_log}
    args="$(envsubst < $script_dir/config/command_args.template | tr '\n' ' ')"
    echo "$script_dir/sleep_until.py $start_time && $program $args"

    peer_id=$((peer_id+1))
done < "$script_dir/config/names.txt" \
    | parallel --timeout "$remote_timeout" --results "$log_dir/{#}"
