import os
import argparse
from sre_parse import expand_template
from matplotlib.pyplot import text
import plotly.graph_objects as go
import pandas as pd
import plotly.express as px

pd.options.plotting.backend = "plotly"


def plot_figs_df(exp_dict, output_dir):
    for exp_key in exp_dict:
        # print(exp_dict[exp_key]["delay"])
        fig = px.box(exp_dict[exp_key], x="delay", y="actual_time", color="delay")
        df = exp_dict[exp_key]
        for s in df.delay.unique():
            # print(str(len(df[df["delay"] == s]["actual_time"])))
            fig.add_annotation(
                x=s,
                y=df[df["delay"] == s]["actual_time"].max(),
                text="count = " + str(len(df[df["delay"] == s]["actual_time"])),
                yshift=10,
                showarrow=False,
            )
        # Update layout
        fig.update_layout(
            title=exp_key,
            xaxis_title="Assigned Sleep Time (ms)",
            yaxis_title="Actual Sleep Time (ms)",
            legend_title="Assigned Sleep Time",
            font=dict(family="arial, monospace", size=18, color="black"),
        )

        fig.write_image(output_dir + "/delay_time_" + str(exp_key) + ".png")
        fig.write_html(output_dir + "/delay_time_" + str(exp_key) + ".html")


def plot_figs(exp_dict, output_dir):
    for exp_key in exp_dict:
        fig = go.Figure()
        for delay in exp_dict[exp_key]:
            fig.add_trace(
                go.Box(
                    y=exp_dict[exp_key][delay],
                    quartilemethod="linear",
                    name=str(delay),
                    text=["count = " + str(len(exp_dict[exp_key][delay]))],
                )
            )
        # Update layout
        fig.update_layout(
            title=exp_key,
            xaxis_title="Assigned Sleep Time (ms)",
            yaxis_title="Actual Sleep Time (ms)",
            legend_title="Assigned Sleep Time",
            font=dict(family="arial, monospace", size=18, color="black"),
        )

        fig.write_image(output_dir + "/delay_time_" + str(exp_key) + ".png")
        fig.write_html(output_dir + "/delay_time_" + str(exp_key) + ".html")


def main(args):
    exp_dict = {}
    with open(args.input_path, "r") as open_file:
        line = open_file.readline()
        while line != "":
            # print("line = ", line)
            # perform line reading here
            if "Namespace" in line and "total_pub_peers" in line:
                total_pub_peers = int(
                    list(filter(lambda str: str != "", line.rstrip().split(",")))[-1]
                    .strip(")")
                    .split("=")[-1]
                )
                if not total_pub_peers in exp_dict.keys():
                    exp_dict[total_pub_peers] = {}
            if (
                "[/home/vkuo/zenoh/zenoh/src/net/runtime/orchestrator.rs:541] delay ="
                in line
            ):
                delay = line.split("= ")[-1].rstrip()
                try:
                    delay = int(delay)
                except:
                    line = open_file.readline()
                    continue
                # print(total_pub_peers)
                # print(delay)
                if not delay in exp_dict[total_pub_peers].keys():
                    exp_dict[total_pub_peers][delay] = []
            if (
                "[/home/vkuo/zenoh/zenoh/src/net/runtime/orchestrator.rs:541] elapsed_time.as_millis() ="
                in line
            ):
                elapsed_time = line.split("= ")[-1].rstrip()
                try:
                    elapsed_time = int(elapsed_time)
                except:
                    line = open_file.readline()
                    continue
                # print(total_pub_peers)
                # print(delay)
                # print(elapsed_time)
                if not delay in exp_dict[total_pub_peers].keys():
                    line = open_file.readline()
                    continue
                else:
                    exp_dict[total_pub_peers][delay].append(elapsed_time)

            line = open_file.readline()
            # break
    # print(exp_dict)
    df_dict = {}
    for total_pub_peers in exp_dict:
        d = dict()
        d["delay"] = []
        d["actual_time"] = []
        for delay in exp_dict[total_pub_peers]:
            for time in exp_dict[total_pub_peers][delay]:
                d["delay"].append(delay)
                d["actual_time"].append(time)
        df = pd.DataFrame(d)
        # print(df.head())
        df_dict[total_pub_peers] = df

    # df = pd.DataFrame(exp_dict[40])
    # print(df_dict)
    # print(exp_dict[39])
    plot_figs_df(df_dict, args.output_dir)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Parse and plot the time distribution of scouting delay."
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
        "--input_path",
        type=str,
        help="The path to the log",
        required=True,
    )

    args = parser.parse_args()
    if not os.path.exists(args.input_path):
        print("Input path {} does not exists".format(args.input_path))
        exit(1)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)

    main(args)
