#!/usr/bin/env bash
set -e

trap 'kill $(jobs -p)' INT

script_dir="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
source "$script_dir/steps/00_config.sh"
source "$script_dir/steps/05_kill_zenohd.sh"

wait
