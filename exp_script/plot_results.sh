#!/usr/bin/env bash

proj_root=$(git rev-parse --show-toplevel)
exp_dir="./exp_logs"
out_dir="./results_plotting"

mkdir -p $out_dir
for payload_dir in $(find . -type d -path './exp_logs/**/test/payload'); do
    for log_dir in $(ls $payload_dir); do
        python3 ${proj_root}/src/parse_exps_payload_ind_tl.py \
            --input_dir ${payload_dir}/${log_dir} \
            --output_dir $out_dir \
            --log_range &
    done
done
wait

echo "All plottings have been stored at ${out_dir}."
