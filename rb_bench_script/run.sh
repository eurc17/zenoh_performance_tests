#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
repo_dir="$script_dir/.."

pushd "$script_dir"

source "$script_dir/steps/00_config.sh"
source "$script_dir/steps/01_compile.sh"
source "$script_dir/steps/02_deploy_files.sh"
# source "$script_dir/steps/03_build_zenohd.sh"
source "$script_dir/steps/04_run_zenohd.sh"
source "$script_dir/steps/06_run_test.sh"
source "$script_dir/steps/05_kill_zenohd.sh"

popd
