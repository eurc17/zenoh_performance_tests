#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e


while read addr port peer_id; do
    {
        # Remove remote dir
        # ssh -p "$port" "pi@$addr" "rm -rf $remote_dir"
        
        # Copy files
        rsync -aPz -e "ssh -p $port" \
              --exclude '.git' \
              --exclude 'exp_script' \
              --exclude 'rb_bench_script/files' \
              --exclude 'cargo_home' \
              --exclude 'target' \
              --exclude 'armv7l-linux-musleabihf-cross' \
              --exclude 'armv7l-linux-musleabihf-cross.tgz' \
              "$repo_dir/" \
              "pi@$addr:$remote_dir" 

        ssh -p "$port" "pi@$addr" "cd $remote_dir && mkdir -p target/release"
        
        rsync -aPz -e "ssh -p $port" \
              "$repo_dir/target/armv7-unknown-linux-musleabihf/release/$binary_name" \
              "pi@$addr:$remote_dir/target/release/$binary_name" 
    } >/dev/null &
done < "$script_dir/config/rpi_addrs.txt"

wait
