import subprocess
import os
import glob
import argparse
import matplotlib.pyplot as plt
import json
from functools import cmp_to_key


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


def plot(exp_dict, output_dir):
    fig, ax = plt.subplots()
    peer_list = []
    recv_rate_list = []
    for exp_key in exp_dict:
        peer_num = int(exp_key.split("-")[0])
        print(exp_key)
        print("Peer num = ", peer_num)
        recv_rate = float(exp_dict[exp_key]["total_recvd_msg_num"]) / float(
            exp_dict[exp_key]["total_expected_msg_num"]
        )
        peer_list.append(peer_num)
        recv_rate_list.append(recv_rate)

    ax.plot(peer_list, recv_rate_list, marker="o")
    ax.set_xlabel("number of peers", fontweight="bold")
    ax.set_ylabel("msg recv rate", fontweight="bold")
    ax.grid(True)

    ax.set_title(
        "Msg recv rate\n",
        fontsize=14,
        fontweight="bold",
    )
    plt.savefig(args.output_dir + "/recv_rate.png")


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    exp_dict = {}

    for json_file in sorted(
        glob.glob(args.input_dir + "/exp_sub_*.json"), key=sort_file_key
    ):
        base_name = os.path.basename(json_file).split(".")[0]
        peer_id = int(base_name.split("_")[2])
        exp_key = base_name.split("_")[3]
        with open(json_file) as json_opened_file:
            data = json.load(json_opened_file)
            recvd_msg_num = float(data["recvd_msg_num"])
            expected_msg_num = float(data["expected_msg_num"])
            if not exp_key in exp_dict.keys():
                exp_dict[exp_key] = {}
                exp_dict[exp_key]["total_recvd_msg_num"] = 0
                exp_dict[exp_key]["total_expected_msg_num"] = 0

            exp_dict[exp_key]["total_recvd_msg_num"] += recvd_msg_num
            exp_dict[exp_key]["total_expected_msg_num"] += expected_msg_num
    plot(exp_dict, args.output_dir)


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

    args = parser.parse_args()

    main(args)
