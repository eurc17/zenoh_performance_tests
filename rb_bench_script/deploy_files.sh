#!/usr/bin/env bash
set -e

binary_name="reliable-broadcast-benchmark"
remote_dir="~/rb-exp"

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
repo_dir="$script_dir/.."

while read addr port; do
    {
        rsync -aPz -e "ssh -p port" \
              --exclude 'exp_script' \
              --exclude 'cargo_home' \
              --exclude 'target' \
              --exclude 'armv7l-linux-musleabihf-cross' \
              --exclude 'armv7l-linux-musleabihf-cross.tgz' \
              "$repo_dir/" \
              "pi@$addr:$remote_dir" 

        ssh -p "$port" "pi@$ip" "cd $remote_dir && mkdir -p target/release"
        
        rsync -aPz -e "ssh -p port" \
              "$script_dir/target/armv7-unknown-linux-musleabihf/release/$binary_name" \
              "pi@$addr:$remote_dir/target/release/$binary_name" 
    } &
done < "$script_dir/rpi_addrs.txt"



wait
