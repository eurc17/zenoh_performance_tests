#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

sleep_for='5 seconds'

read router_addr router_port < "$script_dir/config/router_addr.txt"


for psize in $payload_sizes
do
    echo "Running test for payload_size=$psize"

    # restart zenohd
    zenohd_log_file="~/rb_exp/zenohd_${log_time}-${psize}.txt"
    ssh -p "$router_port" "pi@$router_addr" "pkill zenohd || true"
    ssh -p "$router_port" "pi@$router_addr" \
        "bash -c 'env RUST_LOG=debug ~/rb_exp/zenohd --no-timestamp > ${zenohd_log_file} 2>&1 &'"

    
    start_time=$(date --date="$sleep_for" +%s)
    
    while read addr port peer_id name
    do
        # the binary program
        program="$remote_dir/target/release/$binary_name"

        # the command options derived from command_args.template
        args=$(sed \
            -e "s#PAYLOAD_SIZE#${psize}#" \
            -e "s#OUTPUT_DIR#${remote_output_dir}#" \
            -e "s#PEER_ID#${peer_id}#" \
            "$script_dir/config/command_args.template")
        

        # the command executed on remote
        stdout_file="$remote_dir/rb_${log_time}_${psize}.stdout"
        stderr_file="$remote_dir/rb_${log_time}_${psize}.stderr"

        cmd="$remote_dir/rb_bench_script/sleep_until.py $start_time && \
             export RUST_LOG=${remote_rust_log} && \
             timeout -s KILL $remote_timeout $program $args > $stdout_file 2> $stderr_file"
        
        # run command on remote
        ssh -p "$port" "pi@$addr" "sh -c \"$cmd\"" &
        
    done < "$script_dir/config/rpi_addrs.txt"

    wait
done
