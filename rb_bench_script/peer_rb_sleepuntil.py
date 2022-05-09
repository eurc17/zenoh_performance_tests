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
    print(cur_time)
    if cur_time[2] < 50:
        if cur_time[1] < 59:
            return f"{cur_time[0]}:{cur_time[1]+1}"
        elif cur_time[1] < 23:
            return f"{cur_time[0]+1}:{(cur_time[1]+1) % 60}"
        else:
            return f"{(cur_time[0]+1) %24}:{(cur_time[1]+1) % 60}"
    else:
        if cur_time[1] < 58:
            return f"{cur_time[0]}:{cur_time[1]+2}"
        elif cur_time[1] < 23:
            return f"{cur_time[0]+1}:{(cur_time[1]+2) % 60}"
        else:
            return f"{(cur_time[0]+1) %24}:{(cur_time[1]+2) % 60}"

    # if cur_time[2] > 53:
    #     return 120 - cur_time[2]
    # else:
    #     return 60 - cur_time[2]


def main(args):
    print(args)
    num_msgs_per_peer = args.num_msgs_per_peer
    payload_size = args.payload_size
    round_timeout = args.round_timeout  # ms
    program_timeout = 10 + (args.init_time) / 1000  # s
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    start = time.time()

    # sleep_until = subprocess.run(
    #     ["which", "sleepuntil"], stdout=subprocess.PIPE
    # ).stdout.decode("utf-8")
    sleep_until_time = get_sleep()
    print(sleep_until_time)
    if args.pub_disable:
        disable_string = "--pub-disable"
        if args.sub_disable:
            print("Cannot specify pub_disable and sub_disable at the same time")
    elif args.sub_disable:
        disable_string = "--sub-disable"
    else:
        disable_string = ""

    for peer_id_raw in range(args.total_pub_peers):
        peer_id = args.peer_id_start + peer_id_raw
        file_name = "{}_{}-{}-{}-{}-{}-{}".format(
            peer_id,
            args.total_pub_peers,
            args.total_pub_peers,
            num_msgs_per_peer,
            payload_size,
            round_timeout,
            args.init_time,
        )
        end = time.time()
        if args.locators != "":
            cmd = "sleepuntil {} && ./target/release/reliable-broadcast-benchmark -p {} -a {} -m {} -n {} -t {} -o {} -i {} -d {} {} --locators {} -r {} --pub-interval {}".format(
                sleep_until_time,
                peer_id,
                args.total_pub_peers,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                int(round((end - start) * 1000)),
                disable_string,
                args.locators,
                args.remote_pub_peers,
                args.pub_interval,
            )
        else:
            cmd = "sleepuntil {} && ./target/release/reliable-broadcast-benchmark -p {} -a {} -m {} -n {} -t {} -o {} -i {} -d {} {} -r {} --pub-interval {}".format(
                sleep_until_time,
                peer_id,
                args.total_pub_peers,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                int(round((end - start) * 1000)),
                disable_string,
                args.remote_pub_peers,
                args.pub_interval,
            )
        # print(cmd)
        proc = subprocess.Popen(
            cmd,
            shell=True,
        )
        # cmd = "psrecord {} --plot {}/plot_{}.png --log {}/log_{}.txt --include-children --duration {} --interval 0.01 &".format(
        #     proc.pid,
        #     args.output_dir,
        #     file_name,
        #     args.output_dir,
        #     file_name,
        #     program_timeout,
        # )
        # os.system(cmd)
        # print(proc.pid)
    end = time.time()
    print("Elapsed time = ", end - start)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Start session workers")
    parser.add_argument(
        "-p",
        "--total_pub_peers",
        type=int,
        help="The total number of publisher peers.",
    )
    parser.add_argument(
        "-m",
        "--num_msgs_per_peer",
        type=int,
        help="The number of messages each publisher peer will try to send.",
        default=1,
    )
    parser.add_argument(
        "-n",
        "--payload_size",
        type=int,
        help="The payload size (bytes) of the message.",
        default=8,
    )
    parser.add_argument(
        "-t",
        "--round_timeout",
        type=int,
        help="The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).",
        default=100,
    )
    parser.add_argument(
        "-o",
        "--output_dir",
        type=str,
        help="The path to store the output .json file.",
        default=100,
    )
    parser.add_argument(
        "-i",
        "--init_time",
        type=int,
        help="The initialization time (ms) for starting up futures.",
        default=3000,
    )
    parser.add_argument("--pub_disable", action="store_true")
    parser.add_argument("--sub_disable", action="store_true")
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
        "-r",
        "--remote_pub_peers",
        type=int,
        help="The total number of remote publisher peers",
        default=0,
    )
    parser.add_argument(
        "--pub_interval",
        type=int,
        help="The interval between publishing messages Unit: ms",
        default=1,
    )

    args = parser.parse_args()
    main(args)
