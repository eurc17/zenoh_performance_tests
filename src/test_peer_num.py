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
    for peer_num in range(args.peer_num_start, args.peer_num_end + 1):
        file_name = "{}-{}-{}-{}-{}".format(
            peer_num, peer_num, num_msgs_per_peer, payload_size, round_timeout
        )
        cmd = 'psrecord "./target/release/zenoh_performance_tests -p {} -m {} -n {} -t {} -o {}" --plot {}/plot-{}.png --log {}/log-{}.txt --include-children --duration {}'.format(
            peer_num,
            num_msgs_per_peer,
            payload_size,
            round_timeout,
            args.output_dir,
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


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run multiple zenoh-performance-test with increasing peer numbers"
    )
    parser.add_argument(
        "-o",
        "--output_dir",
        type=str,
        help="The directory to store all output results",
        default="./",
    )
    parser.add_argument(
        "-s",
        "--peer_num_start",
        type=int,
        help="The starting number of peer of the test",
        default=1,
    )
    parser.add_argument(
        "-e",
        "--peer_num_end",
        type=int,
        help="The ending number of peer of the test",
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

    args = parser.parse_args()
    main(args)
