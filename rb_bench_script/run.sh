#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
repo_dir="$script_dir/.."

pushd "$script_dir"

source 00_config.sh
source "$script_dir/01_compile.sh"
source "$script_dir/02_deploy_files.sh"
source "$script_dir/03_build_zenohd.sh"
source "$script_dir/04_run_zenohd.sh"
source "$script_dir/06_run_test.sh"
source "$script_dir/05_kill_zenohd.sh"

popd
