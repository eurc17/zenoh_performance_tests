#!/usr/bin/env python3
import os
import glob
import argparse
import time


def main(args):
    print(args)
    num_msgs_per_peer = args.num_msgs_per_peer
    payload_size = args.payload_size
    round_timeout = args.round_timeout  # ms
    program_timeout = 10  # s
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    for init_time in range(args.init_time_start, args.init_time_end + 1):
        file_name = "{}-{}-{}-{}-{}-{}".format(
            args.peer_num,
            args.peer_num,
            num_msgs_per_peer,
            payload_size,
            round_timeout,
            init_time,
        )
        cmd = 'psrecord "./target/release/zenoh_performance_tests -p {} -m {} -n {} -t {} -o {} -i {}" --plot {}/plot-{}.png --log {}/log-{}.txt --include-children --duration {}'.format(
            args.peer_num,
            num_msgs_per_peer,
            payload_size,
            round_timeout,
            args.output_dir,
            init_time,
            args.output_dir,
            file_name,
            args.output_dir,
            file_name,
            program_timeout,
        )
        print(cmd)
        os.system(cmd)
        # sleep for 10 seconds before running new tests
        time.sleep(10)
        os.system("pkill zenoh_perfornmance_tests")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run multiple zenoh-performance-test with increasing initialization time"
    )
    parser.add_argument(
        "-o",
        "--output_dir",
        type=str,
        help="The directory to store all output results",
        default="./",
    )
    parser.add_argument(
        "-p",
        "--peer_num",
        type=int,
        help="The number of peer of the test",
        default=1,
    )
    parser.add_argument(
        "-m",
        "--num_msgs_per_peer",
        type=int,
        help="The number of messages per peer to send within a round",
        default=1,
    )
    parser.add_argument(
        "-n",
        "--payload_size",
        type=int,
        help="The payload size (in bytes) for each message",
        default=8,
    )
    parser.add_argument(
        "-t",
        "--round_timeout",
        type=int,
        help="The timeout (in ms) for each round",
        default=100,
    )
    parser.add_argument(
        "-a",
        "--init_time_start",
        type=int,
        help="The starting testing length of initialization time (in ms) before the first round",
        default=1000,
    )
    parser.add_argument(
        "-e",
        "--init_time_end",
        type=int,
        help="The ending testing length of initialization time (in ms) before the first round",
        default=1000,
    )

    args = parser.parse_args()
    main(args)
