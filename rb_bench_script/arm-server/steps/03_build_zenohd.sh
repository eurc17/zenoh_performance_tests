#!/bin/false "This script should be sourced in a shell, not executed directly"
set -e

read addr port < "$script_dir/config/router_addr.txt"

pushd "$script_dir"
git clone "$zenoh_git_url" || true

pushd zenoh/zenohd
git checkout "$zenoh_git_commit"

cargo build --profile fast --bin zenohd
rsync -aPz -e "ssh -p $port" \
      ../target/fast/zenohd "pi@$addr:$remote_dir"

popd
popd
