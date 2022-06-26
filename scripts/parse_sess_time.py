from functools import cmp_to_key
import os
import glob
import argparse
import json
import plotly.offline as pyo
import plotly.graph_objs as go


def sort_file(filename1, filename2):
    base_name1 = os.path.basename(filename1).split(".")[0]
    base_name2 = os.path.basename(filename2).split(".")[0]
    peer_id_1 = int(base_name1.split("_")[1])
    peer_id_2 = int(base_name2.split("_")[1])
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


def plot(exp_dict, output_dir, dist=0.15):
    for exp_key in exp_dict:
        fig = go.Figure()
        for peer_id in exp_dict[exp_key]:
            # up to pub_sub_worker_start
            x = exp_dict[exp_key][peer_id]["list_start_timestamp"]
            y = exp_dict[exp_key][peer_id]["list_timestamp_peer_num"]
            fig.add_trace(go.Scatter(x=x, y=y, name=str(peer_id)))

        # Update layout
        fig.update_layout(
            title=exp_key,
            xaxis_title="Time (ms)",
            yaxis_title="Peer num",
            legend_title="Peer ID",
            font=dict(family="arial, monospace", size=18, color="black"),
        )

        fig.write_image(output_dir + "/sess_time_" + exp_key + ".png")
        fig.write_html(output_dir + "/sess_time_" + exp_key + ".html")


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    exp_dict = {}
    for put_info_file in sorted(
        glob.glob(args.input_dir + "/Session_*.json"), key=sort_file_key
    ):
        base_name = os.path.basename(put_info_file).split(".")[0]
        peer_id = int(base_name.split("_")[1])
        exp_key = base_name.split("_")[3]
        with open(put_info_file) as json_opened_file:
            data = json.load(json_opened_file)
            if not exp_key in exp_dict.keys():
                exp_dict[exp_key] = {}
            exp_dict[exp_key][peer_id] = data
    plot(exp_dict, args.output_dir)


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
