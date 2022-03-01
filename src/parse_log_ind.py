from functools import cmp_to_key
import os
import glob
import argparse
import json
from numpy import append
import plotly.offline as pyo
import plotly.graph_objs as go
from plotly.subplots import make_subplots


def sort_file(filename1, filename2):
    base_name1 = os.path.basename(filename1).split(".")[0]
    base_name2 = os.path.basename(filename2).split(".")[0]
    peer_id_1 = int(base_name1.split("_")[1])
    peer_id_2 = int(base_name2.split("_")[1])
    int_list1 = list(map(int, base_name1.split("_")[2].split("-")))
    int_list2 = list(map(int, base_name2.split("_")[2].split("-")))
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


def get_data_from_file(file_path):
    ret = []
    with open(file_path, "r") as open_file:
        line = open_file.readline()
        while line != "":
            # print("line = ", line)
            line = open_file.readline()
            data_vec = list(map(float, filter(None, line.rstrip().split(" "))))
            # print(data_vec)
            if len(data_vec) != 0:
                ret.append(data_vec)
    return ret


def plot_usage(exp_dict, output_dir, dist=0.15):
    for exp_key in exp_dict:
        fig = make_subplots(specs=[[{"secondary_y": True}]])
        # fig = go.Figure()
        total_cpu_usage = []
        total_mem_usage = []
        for peer_id in exp_dict[exp_key]:
            # up to pub_sub_worker_start
            data_peer = exp_dict[exp_key][peer_id]
            for i, data_vec in enumerate(data_peer):
                if i + 1 > len(total_cpu_usage):
                    total_cpu_usage.append(data_vec[1])
                else:
                    total_cpu_usage[i] += data_vec[1]
                if i + 1 > len(total_mem_usage):
                    total_mem_usage.append(data_vec[2])
                else:
                    total_mem_usage[i] += data_vec[2]

        # Create figure with secondary y-axis

        # Add traces
        fig.add_trace(
            go.Scatter(
                x=[i * 0.01 for i in range(0, len(total_cpu_usage))],
                y=total_cpu_usage,
                name="CPU Usage",
            ),
            secondary_y=False,
        )

        fig.add_trace(
            go.Scatter(
                x=[i * 0.01 for i in range(0, len(total_mem_usage))],
                y=total_mem_usage,
                name="Mem Usage",
            ),
            secondary_y=True,
        )

        # # Update layout
        fig.update_layout(
            title=exp_key,
            xaxis_title="Time (ms)",
            legend_title="Utilization",
            font=dict(family="arial, monospace", size=18, color="black"),
        )

        fig.update_yaxes(title_text="CPU Usage (%)", secondary_y=False)
        fig.update_yaxes(title_text="Memory Usage (MB)", secondary_y=True)

        fig.write_image(output_dir + "/plot_usage_" + exp_key + ".png")
        fig.write_html(output_dir + "/plot_usage_" + exp_key + ".html")


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    exp_dict = {}
    for log_info_file in sorted(
        glob.glob(args.input_dir + "/log_*.txt"), key=sort_file_key
    ):
        base_name = os.path.basename(log_info_file).split(".")[0]
        peer_id = int(base_name.split("_")[1])
        exp_key = base_name.split("_")[2]
        if not exp_key in exp_dict.keys():
            exp_dict[exp_key] = {}
        exp_dict[exp_key][peer_id] = get_data_from_file(log_info_file)
    plot_usage(exp_dict, args.output_dir)
    # with open(put_info_file) as json_opened_file:
    #     data = json.load(json_opened_file)
    #     if not exp_key in exp_dict.keys():
    #         exp_dict[exp_key] = {}
    #     exp_dict[exp_key][peer_id] = data


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Parse and plot the time distribution."
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
