#!/bin/false "This script should be sourced in a shell, not executed directly"
read addr port < "$script_dir/config/router_addr.txt"
ssh -p "$port" "pi@$addr" "sh -c 'killall -e zenohd >/dev/null 2>&1 || true'" &

while read addr port peer_id name
do
    ssh -p "$port" "pi@$addr" "killall -e '$binary_name' >/dev/null 2>&1 || true" </dev/null \
        || echo "unable to connect to $addr:$port" &
done < "$script_dir/config/rpi_addrs.txt"
