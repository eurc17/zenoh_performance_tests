#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

while read addr port peer_id name
do
    {
        remote_log_dir="$remote_dir/test/"
        local_log_dir="$script_dir/exp_logs/$name/log_exp_${log_time}/test"
        
        mkdir -p "$local_log_dir"
        rsync -aPz -e "ssh -p $port" \
              "pi@$addr:$remote_log_dir" \
              "$local_log_dir"
    } >/dev/null 2>&1 &
done < "$script_dir/config/rpi_addrs.txt"

wait
