import os
import glob
import argparse

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import json
from functools import cmp_to_key


def get_exp_key(file_base_name):
    split_string = file_base_name.split("-")[0:3] + file_base_name.split("-")[4:]
    return "-".join(split_string)


def sort_file(filename1, filename2):
    base_name1 = os.path.basename(filename1).split(".")[0]
    base_name2 = os.path.basename(filename2).split(".")[0]
    peer_id_1 = int(base_name1.split("_")[2])
    peer_id_2 = int(base_name2.split("_")[2])
    int_list1 = list(map(int, base_name1.split("_")[3].split("-")))
    int_list2 = list(map(int, base_name2.split("_")[3].split("-")))
    if peer_id_1 > peer_id_2:
        return 1
    elif peer_id_1 < peer_id_2:
        return -1
    else:
        for i in range(len(int_list1)):
            if int_list1[i] > int_list2[i]:
                return 1
            elif int_list1[i] < int_list2[i]:
                return -1
    return 0


sort_file_key = cmp_to_key(sort_file)


def plot(exp_dict, args):
    fig, ax = plt.subplots()
    peer_list = []
    throughput_list = []
    latency_list = []
    for exp_key in exp_dict:
        for peer_id in exp_dict[exp_key]:
            peer_list.append(peer_id)
            throughput_list.append(
                exp_dict[exp_key][peer_id]["result_vec"][0]["throughput"]
            )
            latency_list.append(
                exp_dict[exp_key][peer_id]["result_vec"][0]["average_latency_ms"]
            )

    fig, ax1 = plt.subplots()

    color = "tab:red"
    ax1.set_xlabel("payload size (bytes)", fontweight="bold")
    ax1.set_ylabel("throughput (msgs/s)", color=color, fontweight="bold")
    ax1.plot(peer_list, throughput_list, color=color)
    ax1.tick_params(axis="y", labelcolor=color)
    if args.log_range:
        ax1.set_xscale("log")

    ax2 = ax1.twinx()  # instantiate a second axes that shares the same x-axis

    color = "tab:blue"
    ax2.set_ylabel("Average Latency (ms)", color=color, fontweight="bold")
    ax2.plot(peer_list, latency_list, color=color)
    ax2.tick_params(axis="y", labelcolor=color)

    fig.tight_layout()  # otherwise the right y-label is slightly clipped
    plt.savefig(args.output_dir + "/throughput_latency.png")


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    exp_dict = {}

    for json_file in sorted(
        glob.glob(args.input_dir + "/exp_sub_*.json"), key=sort_file_key
    ):
        base_name = os.path.basename(json_file).split(".")[0]
        payload_size = int(base_name.split("-")[3])
        print(payload_size)
        exp_key = base_name.split("_")[3]
        exp_key = get_exp_key(base_name)
        with open(json_file) as json_opened_file:
            data = json.load(json_opened_file)
            if not exp_key in exp_dict.keys():
                exp_dict[exp_key] = {}
            exp_dict[exp_key][payload_size] = data
    plot(exp_dict, args)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Parse the result of running multiple zenoh-performance-test with increasing peer numbers. RUST_LOG=info must be set"
    )
    parser.add_argument(
        "-o",
        "--output_dir",
        type=str,
        help="The directory to store output chart",
        default="./",
    )
    parser.add_argument(
        "-i",
        "--input_dir",
        type=str,
        help="The directory where the logs are stored",
        required=True,
    )
    parser.add_argument(
        "--log_range",
        action="store_true",
        help="Step the payload size in log scale",
    )

    args = parser.parse_args()

    main(args)
