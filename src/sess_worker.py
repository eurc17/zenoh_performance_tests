import os
import glob
import argparse
import time
import subprocess


def main(args):
    print(args)
    num_msgs_per_peer = args.num_msgs_per_peer
    payload_size = args.payload_size
    round_timeout = args.round_timeout  # ms
    program_timeout = 10 + (args.init_time) / 1000  # s
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    start = time.time()

    sleep_until = subprocess.run(
        ["which", "sleepuntil"], stdout=subprocess.PIPE
    ).stdout.decode("utf-8")

    if sleep_until == "" or True:
        for peer_id in range(args.total_pub_peers):
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
            cmd = "./target/release/session-test-worker -p {} -a {} -m {} -n {} -t {} -o {} -i {} -d {}".format(
                peer_id,
                args.total_pub_peers,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
                int(round((end - start) * 1000)),
            )
            # print(cmd)
            proc = subprocess.Popen(
                cmd,
                shell=True,
            )
            cmd = "psrecord {} --plot {}/plot_{}.png --log {}/log_{}.txt --include-children --duration {} &".format(
                proc.pid,
                args.output_dir,
                file_name,
                args.output_dir,
                file_name,
                program_timeout,
            )
            os.system(cmd)
            # print(proc.pid)
        end = time.time()
        print("Elapsed time = ", end - start)
    else:
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
        if cur_time[2] > 55:
            cur_time[2] = 0
            if cur_time[1] < 58:
                cur_time[1] += 2
            else:
                cur_time[1] = (cur_time[1] + 2) % 60
                if cur_time[0] != 23:
                    cur_time[0] += 1
                else:
                    cur_time[0] = 0
        else:
            cur_time[2] = 0
            if cur_time[1] != 59:
                cur_time[1] += 1
            else:
                cur_time[1] = 0
                if cur_time[0] != 23:
                    cur_time[0] += 1
                else:
                    cur_time[0] = 0
        for peer_id in range(args.total_pub_peers):
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
            cmd = "sleepuntil {}:{} && ./target/release/session-test-worker -p {} -a {} -m {} -n {} -t {} -o {} -i {}".format(
                cur_time[0],
                cur_time[1],
                peer_id,
                args.total_pub_peers,
                num_msgs_per_peer,
                payload_size,
                round_timeout,
                args.output_dir,
                args.init_time,
            )
            # print(cmd)
            proc = subprocess.Popen(
                cmd,
                shell=True,
            )
            cmd = "psrecord {} --plot {}/plot_{}.png --log {}/log_{}.txt --include-children --duration {} &".format(
                proc.pid,
                args.output_dir,
                file_name,
                args.output_dir,
                file_name,
                program_timeout,
            )
            os.system(cmd)
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

    args = parser.parse_args()
    main(args)
