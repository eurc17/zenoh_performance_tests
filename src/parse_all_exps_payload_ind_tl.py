import os
import glob
import argparse


def main(args):
    if args.log_range:
        log_range_str = "--log_range"
    else:
        log_range_str = ""
    for zenoh_device_dir in sorted(glob.glob(args.input_dir + "/*/")):
        # print(zenoh_device_dir)
        for exp_dir in sorted(glob.glob(zenoh_device_dir + "/*/test/payload/*/")):
            os.system(
                "python3 ./src/parse_exps_payload_ind_tl.py -i "
                + exp_dir
                + " -o "
                + exp_dir
                + "/graphs "
                + log_range_str
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Parse the multiple results under exp_logs directory. Output graphs will be stored under `graphs` directory under each log directory"
    )
    parser.add_argument(
        "-i",
        "--input_dir",
        type=str,
        help="The path to exp_logs",
        required=True,
    )
    parser.add_argument(
        "--log_range",
        action="store_true",
        help="Step the payload size in log scale",
    )

    args = parser.parse_args()

    main(args)
