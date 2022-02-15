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
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            # up to pub_sub_worker_start
            x.extend([0, exp_dict[exp_key][peer_id]["pub_sub_worker_start"], None])
            y.extend([peer_id, peer_id, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="pub_sub_worker_start"))
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            # up to session_start
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["pub_sub_worker_start"],
                    exp_dict[exp_key][peer_id]["session_start"],
                    None,
                ]
            )
            y.extend([peer_id, peer_id, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="session_start"))

        # up to start_pub_worker
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["session_start"],
                    exp_dict[exp_key][peer_id]["start_pub_worker"],
                    None,
                ]
            )
            y.extend([peer_id + dist, peer_id + dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="start_pub_worker"))

        # up to before_sending
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["start_pub_worker"],
                    exp_dict[exp_key][peer_id]["before_sending"],
                    None,
                ]
            )
            y.extend([peer_id + dist, peer_id + dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="before_sending"))

        # up to start_sending
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["before_sending"],
                    exp_dict[exp_key][peer_id]["start_sending"],
                    None,
                ]
            )
            y.extend([peer_id + dist, peer_id + dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="start_sending"))

        # up to after_sending
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["start_sending"],
                    exp_dict[exp_key][peer_id]["after_sending"],
                    None,
                ]
            )
            y.extend([peer_id + dist, peer_id + dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="after_sending"))

        # up to start_sub_worker
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["session_start"],
                    exp_dict[exp_key][peer_id]["start_sub_worker"],
                    None,
                ]
            )
            y.extend([peer_id - dist, peer_id - dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="start_sub_worker"))

        # up to after_subscribing
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["start_sub_worker"],
                    exp_dict[exp_key][peer_id]["after_subscribing"],
                    None,
                ]
            )
            y.extend([peer_id - dist, peer_id - dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="after_subscribing"))

        # up to start_receiving
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["after_subscribing"],
                    exp_dict[exp_key][peer_id]["start_receiving"],
                    None,
                ]
            )
            y.extend([peer_id - dist, peer_id - dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="start_receiving"))

        # up to after_receiving
        x = []
        y = []
        for peer_id in exp_dict[exp_key]:
            x.extend(
                [
                    exp_dict[exp_key][peer_id]["start_receiving"],
                    exp_dict[exp_key][peer_id]["after_receiving"],
                    None,
                ]
            )
            y.extend([peer_id - dist, peer_id - dist, None])
        fig.add_trace(go.Scatter(x=x, y=y, name="after_receiving"))

        fig.write_image(output_dir + "/" + exp_key + ".png")
        fig.write_html(output_dir + "/" + exp_key + ".html")


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    exp_dict = {}
    for put_info_file in sorted(
        glob.glob(args.input_dir + "/put_*.json"), key=sort_file_key
    ):
        base_name = os.path.basename(put_info_file).split(".")[0]
        peer_id = int(base_name.split("_")[1])
        exp_key = base_name.split("_")[3]
        with open(put_info_file) as json_opened_file:
            data = json.load(json_opened_file)
            if not exp_key in exp_dict.keys():
                exp_dict[exp_key] = {}
            exp_dict[exp_key][peer_id] = data
    for sub_info_file in sorted(
        glob.glob(args.input_dir + "/sub_*.json"), key=sort_file_key
    ):
        base_name = os.path.basename(sub_info_file).split(".")[0]
        peer_id = int(base_name.split("_")[1])
        exp_key = base_name.split("_")[3]
        with open(sub_info_file) as json_opened_file:
            data = json.load(json_opened_file)
            if not exp_key in exp_dict.keys():
                exp_dict[exp_key] = {}
            if not peer_id in exp_dict[exp_key]:
                exp_dict[exp_key][peer_id] = data
            else:
                exp_dict[exp_key][peer_id] = {**exp_dict[exp_key][peer_id], **data}
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
