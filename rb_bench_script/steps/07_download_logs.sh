#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

while read addr port peer_id name
do
    {
        echo $addr $port
        local_dir="$script_dir/exp_logs/$name/log_exp_${log_time}/$output_dir_suffix"
        
        mkdir -p "$local_dir"
        rsync -aPz -e "ssh -p $port" \
              "pi@$addr:$remote_output_dir/" \
              "$local_dir"
    } &
done < "$script_dir/config/rpi_addrs.txt"

wait
