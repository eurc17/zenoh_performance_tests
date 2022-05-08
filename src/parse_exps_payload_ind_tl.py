import os
import glob
import argparse

import matplotlib

# matplotlib.use("Agg")
# import matplotlib.pyplot as plt
import json
from functools import cmp_to_key
import plotly.graph_objs as go
from operator import add


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
    payload_list = []
    throughput_dict = {}
    latency_dict = {}
    peer_name_list = [
        "zenoh-2-1",
        "zenoh-2-2",
        "zenoh-2-3",
        "zenoh-2-4",
        "zenoh-2-5",
        "zenoh-2-6",
        "zenoh-3-1",
        "zenoh-3-2",
        "zenoh-3-3",
        "zenoh-3-4",
        "zenoh-3-5",
        "zenoh-3-6",
    ]
    for exp_key in exp_dict:
        fig_throughput = go.Figure()
        fig_latency = go.Figure()
        for payload_size in exp_dict[exp_key]:
            # print(payload_size)
            payload_list.append(payload_size)
            for result in exp_dict[exp_key][payload_size]["result_vec"]:
                peer_id = int(result["key_expr"].split("/")[-1])
                if peer_id not in throughput_dict:
                    throughput_dict[peer_id] = []
                if peer_id not in latency_dict:
                    latency_dict[peer_id] = []
                throughput_dict[peer_id].append(result["throughput"])
                latency_dict[peer_id].append(result["average_latency_ms"])
        all_peer_ids = list(range(0, 12))
        # plot throughput from peers
        for peer_id in throughput_dict:
            fig_throughput.add_trace(
                go.Scatter(
                    x=payload_list,
                    y=throughput_dict[peer_id],
                    name=peer_name_list[peer_id],
                )
            )
            all_peer_ids.remove(peer_id)
        for peer_id in latency_dict:
            fig_latency.add_trace(
                go.Scatter(
                    x=payload_list,
                    y=latency_dict[peer_id],
                    name=peer_name_list[peer_id],
                )
            )
        # plot average throughput
        # get average throughput
        zenoh_2_throughput_sum = [0] * len(exp_dict[exp_key])
        zenoh_2_throughput_cnt = 0
        zenoh_3_throughput_sum = [0] * len(exp_dict[exp_key])
        zenoh_3_throughput_cnt = 0
        # print("zenoh_2_throughput_sum = ", zenoh_2_throughput_sum)
        for peer_id in throughput_dict:
            if peer_id < 6:
                zenoh_2_throughput_sum = list(
                    map(add, throughput_dict[peer_id], zenoh_2_throughput_sum)
                )
                zenoh_2_throughput_cnt += 1
            else:
                zenoh_3_throughput_sum = list(
                    map(add, throughput_dict[peer_id], zenoh_3_throughput_sum)
                )
                zenoh_3_throughput_cnt += 1
        zenoh_2_throughput = []
        zenoh_3_throughput = []
        zenoh_total_throughput_sum = list(
            map(add, zenoh_2_throughput_sum, zenoh_3_throughput_sum)
        )
        zenoh_total_throughput_cnt = zenoh_2_throughput_cnt + zenoh_3_throughput_cnt
        zenoh_total_throughput = []
        for throughput_sum in zenoh_2_throughput_sum:
            zenoh_2_throughput.append(throughput_sum / zenoh_2_throughput_cnt)
        for throughput_sum in zenoh_3_throughput_sum:
            zenoh_3_throughput.append(throughput_sum / zenoh_3_throughput_cnt)
        for throughput_sum in zenoh_total_throughput_sum:
            zenoh_total_throughput.append(throughput_sum / zenoh_total_throughput_cnt)
        fig_throughput.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_2_throughput,
                name="zenoh-2-average",
            )
        )
        fig_throughput.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_3_throughput,
                name="zenoh-3-average",
            )
        )
        fig_throughput.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_total_throughput,
                name="zenoh-3-average",
            )
        )

        # for latency
        # plot average latency
        # get average latency
        zenoh_2_latency_sum = [0] * len(exp_dict[exp_key])
        zenoh_2_latency_cnt = 0
        zenoh_3_latency_sum = [0] * len(exp_dict[exp_key])
        zenoh_3_latency_cnt = 0
        # print("zenoh_2_throughput_sum = ", zenoh_2_throughput_sum)
        for peer_id in latency_dict:
            if peer_id < 6:
                zenoh_2_latency_sum = list(
                    map(add, latency_dict[peer_id], zenoh_2_latency_sum)
                )
                zenoh_2_latency_cnt += 1
            else:
                zenoh_3_latency_sum = list(
                    map(add, latency_dict[peer_id], zenoh_3_latency_sum)
                )
                zenoh_3_latency_cnt += 1
        zenoh_2_latency = []
        zenoh_3_latency = []
        zenoh_total_latency_sum = list(
            map(add, zenoh_2_latency_sum, zenoh_3_latency_sum)
        )
        zenoh_total_latency_cnt = zenoh_2_latency_cnt + zenoh_3_latency_cnt
        zenoh_total_latency = []
        for latency_sum in zenoh_2_latency_sum:
            zenoh_2_latency.append(latency_sum / zenoh_2_latency_cnt)
        for latency_sum in zenoh_3_latency_sum:
            zenoh_3_latency.append(latency_sum / zenoh_3_latency_cnt)
        for latency_sum in zenoh_total_latency_sum:
            zenoh_total_latency.append(latency_sum / zenoh_total_latency_cnt)
        fig_latency.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_2_latency,
                name="zenoh-2-average",
            )
        )
        fig_latency.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_3_latency,
                name="zenoh-3-average",
            )
        )
        fig_latency.add_trace(
            go.Scatter(
                x=payload_list,
                y=zenoh_total_latency,
                name="zenoh-3-average",
            )
        )
        # Update layout
        self_peer_id = all_peer_ids[0]
        # print(self_peer_id)
        fig_throughput.update_layout(
            title="Throughput observed from peer {}".format(
                peer_name_list[self_peer_id]
            ),
            xaxis_title="Payload size (bytes)",
            yaxis_title="Throughput (msgs/s)",
            legend_title="Peer measured from",
            font=dict(family="arial, monospace", size=18, color="black"),
        )
        if args.log_range:
            fig_throughput.update_xaxes(type="log")
        fig_throughput.write_image(
            args.output_dir
            + "/throughput_"
            + peer_name_list[self_peer_id]
            + "_"
            + exp_key
            + ".png"
        )
        fig_throughput.write_html(
            args.output_dir
            + "/throughput_"
            + peer_name_list[self_peer_id]
            + "_"
            + exp_key
            + ".html"
        )
        # for latency
        fig_latency.update_layout(
            title="Latency observed from peer {}".format(peer_name_list[self_peer_id]),
            xaxis_title="Payload size (bytes)",
            yaxis_title="Latency (ms)",
            legend_title="Peer measured from",
            font=dict(family="arial, monospace", size=18, color="black"),
        )
        if args.log_range:
            fig_latency.update_xaxes(type="log")
        fig_latency.write_image(
            args.output_dir
            + "/latency_"
            + peer_name_list[self_peer_id]
            + "_"
            + exp_key
            + ".png"
        )
        fig_latency.write_html(
            args.output_dir
            + "/latency_"
            + peer_name_list[self_peer_id]
            + "_"
            + exp_key
            + ".html"
        )


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
        # print(payload_size)
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
