#!/usr/bin/env python3
import os
import glob
import argparse
import time
import subprocess
import numpy


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
    if cur_time[2] > 50:
        return 120 - cur_time[2]
    else:
        return 60 - cur_time[2]


def main(args):
    print(args)
    num_msgs_per_peer = args.num_msgs_per_peer
    # payload_size = args.payload_size
    peer_num = args.peer_num
    round_timeout = args.round_timeout  # ms
    startup_delay = max((10 * args.peer_num) / 1000, 2)
    program_timeout = (
        round_timeout / 1000 + 10 + (args.init_time) / 1000 + startup_delay
    )  # s
    sleep_until = subprocess.run(
        ["which", "sleepuntil"], stdout=subprocess.PIPE
    ).stdout.decode("utf-8")
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    if args.pub_disable:
        disable_string = "--pub_disable"
        if args.sub_disable:
            print("Cannot specify pub_disable and sub_disable at the same time")
    elif args.sub_disable:
        disable_string = "--sub_disable"
    else:
        disable_string = ""
    if args.log_range:
        payload_size = args.payload_size_start
        while payload_size < args.payload_size_end + 1:
            if sleep_until == "" or True:
                actual_program_timeout = program_timeout + get_sleep()
            else:
                actual_program_timeout = program_timeout + get_sleep()
            cmd = "python3 ./src/peer_worker_sleepuntil.py -p {} -m {} -n {} -t {} -o {} -i {} {} --peer_id_start {} --locators {}".format(
                peer_num,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                disable_string,
                args.peer_id_start,
                args.locators,
            )
            # print(cmd)
            # os.system(cmd)
            proc = subprocess.Popen(
                cmd,
                shell=True,
            )
            # sleep for 10 seconds before running new tests
            print(actual_program_timeout)
            time.sleep(actual_program_timeout)
            os.system("pkill pub-sub-worker")
            time.sleep(1)
            payload_size *= args.payload_size_step
    else:

        for payload_size in range(
            args.payload_size_start, args.payload_size_end + 1, args.payload_size_step
        ):
            if sleep_until == "" or True:
                actual_program_timeout = program_timeout + get_sleep()
            else:
                actual_program_timeout = program_timeout + get_sleep()
            cmd = "python3 ./src/peer_worker_sleepuntil.py -p {} -m {} -n {} -t {} -o {} -i {} {} --peer_id_start {} --locators {}".format(
                peer_num,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                disable_string,
                args.peer_id_start,
                args.locators,
            )
            # print(cmd)
            # os.system(cmd)
            proc = subprocess.Popen(
                cmd,
                shell=True,
            )
            # sleep for 10 seconds before running new tests
            print(actual_program_timeout)
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
        "--payload_size_start",
        type=int,
        help="The starting number of payload size (in bytes) for each message",
        default=1,
    )
    parser.add_argument(
        "-e",
        "--payload_size_end",
        type=int,
        help="The ending number of payload size (in bytes) for each message",
        default=1,
    )
    parser.add_argument(
        "-s",
        "--payload_size_step",
        type=int,
        help="The step of payload size (in bytes) for each message",
        default=1,
    )
    parser.add_argument(
        "-m",
        "--num_msgs_per_peer",
        type=int,
        help="The number of messages per peer to send within a round",
        default=1_000_000,
    )
    parser.add_argument(
        "-n",
        "--peer_num",
        type=int,
        help="The total peer number of the test",
        default=1,
    )
    parser.add_argument(
        "-t",
        "--round_timeout",
        type=int,
        help="The timeout (in ms) for each round",
        default=5000,
    )
    parser.add_argument(
        "-i",
        "--init_time",
        type=int,
        help="The initialization time (in ms) before the first round",
        default=3000,
    )
    parser.add_argument(
        "--pub_disable",
        action="store_true",
        help="Disable publisher in all peers spawned by this script",
    )
    parser.add_argument(
        "--sub_disable",
        action="store_true",
        help="Disable subscriber in all peers spawned by this script",
    )
    parser.add_argument(
        "--peer_id_start",
        type=int,
        help="The starting number of the Peer ID",
        default=0,
    )
    parser.add_argument(
        "--locators",
        type=str,
        help="Specifies locators for each peer to connect to (example format: tcp/x.x.x.x:7447).",
        default="",
    )
    parser.add_argument(
        "--log_range",
        action="store_true",
        help="Step the payload size in log scale",
    )

    args = parser.parse_args()
    main(args)
