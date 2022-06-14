#!/usr/bin/env python3
import os
import glob
import argparse
import time
import subprocess


def get_sleep():
    cur_time = list(
        map(
            int,
            subprocess.run(["date", '+"%T"'], stdout=subprocess.PIPE)
            .stdout.decode("utf-8")
            .rstrip()
            .strip('"')
            .split(":"),
        )
    )
    if cur_time[2] > 53:
        return 120 - cur_time[2]
    else:
        return 60 - cur_time[2]


def main(args):
    print(args)
    num_msgs_per_peer = args.num_msgs_per_peer
    payload_size = args.payload_size
    round_timeout = args.round_timeout  # ms
    startup_delay = max((10 * args.peer_num_end) / 1000, 2)
    program_timeout = (
        10 + (args.init_time + args.round_timeout) / 1000 + startup_delay
    )  # s
    sleep_until = subprocess.run(
        ["which", "sleepuntil"], stdout=subprocess.PIPE
    ).stdout.decode("utf-8")
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    for peer_num in range(args.peer_num_start, args.peer_num_end + 1):
        if sleep_until == "" or True:
            actual_program_timeout = program_timeout
        else:
            actual_program_timeout = program_timeout + get_sleep()
        # file_name = "{}-{}-{}-{}-{}-{}".format(
        #     peer_num,
        #     peer_num,
        #     num_msgs_per_peer,
        #     payload_size,
        #     round_timeout,
        #     args.init_time,
        # )
        if args.rx_buffer_size is not None:
            cmd = "python3 ./src/peer_worker.py -p {} -m {} -n {} -t {} -o {} -i {} -r {}".format(
                peer_num,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                args.rx_buffer_size,
            )
        else:
            cmd = "python3 ./src/peer_worker.py -p {} -m {} -n {} -t {} -o {} -i {}".format(
                peer_num,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
            )
        # print(cmd)
        # os.system(cmd)
        proc = subprocess.Popen(
            cmd,
            shell=True,
        )
        # sleep for 10 seconds before running new tests
        time.sleep(actual_program_timeout)
        os.system("pkill pub-sub-worker")
        time.sleep(1)


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
        "-a",
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
    parser.add_argument(
        "-i",
        "--init_time",
        type=int,
        help="The initialization time (in ms) before the first round",
        default=1000,
    )
    parser.add_argument(
        "-r",
        "--rx_buffer_size",
        type=int,
        help="The size (Bytes) of the transport link rx buffer",
    )

    args = parser.parse_args()
    main(args)
