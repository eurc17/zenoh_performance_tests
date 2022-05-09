#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

binary_name="reliable-broadcast-benchmark"
remote_dir="/home/pi/rb-exp"
zenoh_git_url="https://github.com/eclipse-zenoh/zenoh.git"
zenoh_git_commit="90539129b1a7c9e8c7d7daaa84138d093f71fedf"
log_time="$(date +'%Y-%m-%d-%T')"
payload_sizes=$(cat "$repo_dir/scripts/exp_payload_list.txt")
remote_output_dir="$remote_dir/test/payload/$log_time"
