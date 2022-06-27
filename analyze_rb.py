#!/usr/bin/env python3

from pathlib import Path
import plotly.express as px
import pandas as pd
import json
import os


DIR_LIST = [
    #  './rb/2022-05-11 16:52:16+08:00_round-50_echo-1',
    #  './rb/2022-05-11 16:36:54+08:00_round-50_echo-20',
    #  './rb/2022-05-11 16:57:56+08:00_round-100_echo-40',
    #  './rb/2022-05-11 21:42:43+08:00_round-50_echo-20_1',
    #  './rb/2022-05-12T19:24:29+08:00_round-20_echo-1_1',
    #  './rb/2022-05-12T19:52:21+08:00_round-50_echo-20_1_10pi',
    #  './rb/2022-05-12T19:55:09+08:00_round-50_echo-40_1_10pi',
    #  './rb/2022-05-12T19:57:30+08:00_round-100_echo-20_1_10pi',
    #  './rb/2022-05-12T22:00:58+08:00_round-100_echo-40_1_10pi',
    #  './rb/2022-05-12T22:03:02+08:00_round-100_echo-50_1_10pi',
    #  './rb/2022-05-12T22:05:18+08:00_round-30_echo-20_1_10pi',
    "./rb/2022-05-18T09:02:58+08:00_pi5x5_round-20_echo-10_payload-128",
    "./rb/2022-05-13T16:15:11+08:00_pi5x5_round-20_echo-10_payload-256",
    "./rb/2022-05-13T16:16:09+08:00_pi5x5_round-20_echo-10_payload-512",
    "./rb/2022-05-13T16:17:07+08:00_pi5x5_round-20_echo-10_payload-1024",
    "./rb/2022-05-13T16:18:04+08:00_pi5x5_round-20_echo-10_payload-2048",
    "./rb/2022-05-13T16:19:01+08:00_pi5x5_round-20_echo-10_payload-4096",
    "./rb/2022-05-16T18:56:28+08:00_pi5x5_round-20_echo-10_payload-8192",
    "./rb/2022-05-13T16:20:59+08:00_pi5x5_round-30_echo-10_payload-128",
    "./rb/2022-05-13T16:21:56+08:00_pi5x5_round-30_echo-10_payload-256",
    "./rb/2022-05-16T21:24:51+08:00_pi5x5_round-30_echo-10_payload-512",
    "./rb/2022-05-17T14:33:58+08:00_pi5x5_round-30_echo-10_payload-1024",
    "./rb/2022-05-16T21:01:20+08:00_pi5x5_round-30_echo-10_payload-2048",
    "./rb/2022-05-16T21:15:21+08:00_pi5x5_round-30_echo-10_payload-4096",
    "./rb/2022-05-16T21:03:04+08:00_pi5x5_round-30_echo-10_payload-8192",
    "./rb/2022-05-13T16:27:57+08:00_pi5x5_round-30_echo-20_payload-128",
    "./rb/2022-05-13T16:28:55+08:00_pi5x5_round-30_echo-20_payload-256",
    "./rb/2022-05-17T14:11:19+08:00_pi5x5_round-30_echo-20_payload-512",
    "./rb/2022-05-17T14:12:06+08:00_pi5x5_round-30_echo-20_payload-1024",
    "./rb/2022-05-13T16:31:43+08:00_pi5x5_round-30_echo-20_payload-2048",
    "./rb/2022-05-13T16:32:55+08:00_pi5x5_round-30_echo-20_payload-4096",
    "./rb/2022-05-13T16:33:56+08:00_pi5x5_round-30_echo-20_payload-8192",
    "./rb/2022-05-13T16:34:54+08:00_pi5x5_round-50_echo-10_payload-128",
    "./rb/2022-05-13T16:35:55+08:00_pi5x5_round-50_echo-10_payload-256",
    "./rb/2022-05-13T16:36:53+08:00_pi5x5_round-50_echo-10_payload-512",
    "./rb/2022-05-17T14:36:14+08:00_pi5x5_round-50_echo-10_payload-1024",
    "./rb/2022-05-17T07:21:16+08:00_pi5x5_round-50_echo-10_payload-2048",
    "./rb/2022-05-13T16:39:53+08:00_pi5x5_round-50_echo-10_payload-4096",
    "./rb/2022-05-13T16:40:53+08:00_pi5x5_round-50_echo-10_payload-8192",
    "./rb/2022-05-13T16:41:51+08:00_pi5x5_round-50_echo-20_payload-128",
    "./rb/2022-05-13T16:42:49+08:00_pi5x5_round-50_echo-20_payload-256",
    "./rb/2022-05-17T07:22:06+08:00_pi5x5_round-50_echo-20_payload-512",
    "./rb/2022-05-13T16:44:47+08:00_pi5x5_round-50_echo-20_payload-1024",
    "./rb/2022-05-18T08:58:45+08:00_pi5x5_round-50_echo-20_payload-2048",
    "./rb/2022-05-13T16:46:20+08:00_pi5x5_round-50_echo-20_payload-4096",
    "./rb/2022-05-17T14:16:08+08:00_pi5x5_round-50_echo-20_payload-8192",
    "./rb/2022-05-17T16:40:03+08:00_pi5x5_round-50_echo-30_payload-128",
    "./rb/2022-05-17T16:40:55+08:00_pi5x5_round-50_echo-30_payload-256",
    "./rb/2022-05-17T16:41:43+08:00_pi5x5_round-50_echo-30_payload-512",
    "./rb/2022-05-17T16:42:30+08:00_pi5x5_round-50_echo-30_payload-1024",
    "./rb/2022-05-13T16:52:23+08:00_pi5x5_round-50_echo-30_payload-2048",
    "./rb/2022-05-13T16:53:00+08:00_pi5x5_round-50_echo-30_payload-4096",
    "./rb/2022-05-13T16:53:00+08:00_pi5x5_round-50_echo-30_payload-4096",
    "./rb/2022-05-18T08:43:41+08:00_pi5x5_round-50_echo-30_payload-8192",
    "./rb/2022-05-17T14:21:01+08:00_pi5x5_round-50_echo-40_payload-128",
    "./rb/2022-05-17T09:28:58+08:00_pi5x5_round-50_echo-40_payload-256",
    "./rb/2022-05-13T16:56:55+08:00_pi5x5_round-50_echo-40_payload-512",
    "./rb/2022-05-13T16:57:53+08:00_pi5x5_round-50_echo-40_payload-1024",
    "./rb/2022-05-13T16:58:52+08:00_pi5x5_round-50_echo-40_payload-2048",
    "./rb/2022-05-13T16:59:54+08:00_pi5x5_round-50_echo-40_payload-4096",
    "./rb/2022-05-13T17:00:51+08:00_pi5x5_round-50_echo-40_payload-8192",
    "./rb/2022-05-13T17:01:49+08:00_pi5x5_round-100_echo-10_payload-128",
    "./rb/2022-05-17T14:22:52+08:00_pi5x5_round-100_echo-10_payload-256",
    "./rb/2022-05-13T17:03:51+08:00_pi5x5_round-100_echo-10_payload-512",
    "./rb/2022-05-18T09:05:24+08:00_pi5x5_round-100_echo-10_payload-1024",
    "./rb/2022-05-13T17:05:48+08:00_pi5x5_round-100_echo-10_payload-2048",
    "./rb/2022-05-17T14:24:43+08:00_pi5x5_round-100_echo-10_payload-4096",
    "./rb/2022-05-13T17:07:54+08:00_pi5x5_round-100_echo-10_payload-8192",
    "./rb/2022-05-17T16:48:42+08:00_pi5x5_round-100_echo-20_payload-128",
    "./rb/2022-05-13T17:09:53+08:00_pi5x5_round-100_echo-20_payload-256",
    "./rb/2022-05-13T17:10:51+08:00_pi5x5_round-100_echo-20_payload-512",
    "./rb/2022-05-13T17:11:49+08:00_pi5x5_round-100_echo-20_payload-1024",
    "./rb/2022-05-13T17:12:46+08:00_pi5x5_round-100_echo-20_payload-2048",
    "./rb/2022-05-16T19:33:26+08:00_pi5x5_round-100_echo-20_payload-4096",
    "./rb/2022-05-16T21:22:04+08:00_pi5x5_round-100_echo-20_payload-8192",
    "./rb/2022-05-18T09:06:26+08:00_pi5x5_round-100_echo-30_payload-128",
    "./rb/2022-05-13T17:16:56+08:00_pi5x5_round-100_echo-30_payload-256",
    "./rb/2022-05-13T17:17:55+08:00_pi5x5_round-100_echo-30_payload-512",
    "./rb/2022-05-13T17:18:53+08:00_pi5x5_round-100_echo-30_payload-1024",
    "./rb/2022-05-13T17:19:51+08:00_pi5x5_round-100_echo-30_payload-2048",
    "./rb/2022-05-13T17:20:40+08:00_pi5x5_round-100_echo-30_payload-4096",
    "./rb/2022-05-13T17:20:40+08:00_pi5x5_round-100_echo-30_payload-4096",
    "./rb/2022-05-16T19:43:27+08:00_pi5x5_round-100_echo-30_payload-8192",
    "./rb/2022-05-13T17:22:38+08:00_pi5x5_round-100_echo-40_payload-128",
    "./rb/2022-05-13T17:23:36+08:00_pi5x5_round-100_echo-40_payload-256",
    "./rb/2022-05-13T17:24:36+08:00_pi5x5_round-100_echo-40_payload-512",
    "./rb/2022-05-17T16:49:31+08:00_pi5x5_round-100_echo-40_payload-1024",
    "./rb/2022-05-17T09:33:55+08:00_pi5x5_round-100_echo-40_payload-2048",
    "./rb/2022-05-17T14:28:43+08:00_pi5x5_round-100_echo-40_payload-4096",
    "./rb/2022-05-13T17:28:23+08:00_pi5x5_round-100_echo-40_payload-8192",
    "./rb/2022-05-13T18:01:14+08:00_round-50_pi5x5_echo-5_payload-128",
    "./rb/2022-05-13T18:02:13+08:00_round-50_pi5x5_echo-5_payload-256",
    "./rb/2022-05-13T18:03:10+08:00_round-50_pi5x5_echo-5_payload-512",
    "./rb/2022-05-13T18:04:08+08:00_round-50_pi5x5_echo-5_payload-1024",
    "./rb/2022-05-13T18:08:34+08:00_round-50_pi5x5_echo-5_payload-2048",
    "./rb/2022-05-13T18:06:08+08:00_round-50_pi5x5_echo-5_payload-4096",
    "./rb/2022-05-13T18:07:06+08:00_round-50_pi5x5_echo-5_payload-8192",
]
OUTPUT_DIR = "analysis_rb"
png_dir = Path(OUTPUT_DIR) / "png"
svg_dir = Path(OUTPUT_DIR) / "svg"
os.makedirs(png_dir, exist_ok=True)
os.makedirs(svg_dir, exist_ok=True)


def mean(lst):
    if len(lst) == 0:
        return None
    return sum(lst) / len(lst)


def parse_data(exp_json: Path, round_interval: int, batched_echo_interval: int):
    with open(exp_json) as f:
        data = json.load(f)

    return dict(
        peer_id=data["short_config"]["peer_id"],
        payload=data["short_config"]["payload_size"],
        round_interval=round_interval,
        batched_echo_interval=batched_echo_interval,
        receive_rate=data["receive_rate"] * 100,
        rb_rounds=data["average_rb_rounds"],
        latency=mean([res["average_latency_ms"] for res in data["result_vec"]]),
        throughput=mean([res["throughput"] for res in data["result_vec"]]),
    )


def load_data(exp_dir: str):
    assert os.path.isdir(exp_dir)
    round_interval = int(exp_dir.split("round-")[-1].split("_")[0])
    batched_echo_interval = int(exp_dir.split("echo-")[-1].split("_")[0])
    df = pd.DataFrame(
        [
            parse_data(exp_json, round_interval, batched_echo_interval)
            for exp_json in Path(exp_dir).glob("**/test/payload/*/exp_sub_*.json")
        ]
    )

    #  convert throughput in MiB / sec
    df["throughput"] *= df["payload"] / 1024.0 / 1024.0
    #  df['throughput'] *= df['payload']

    #  # convert latency in second
    #  df['latency'] /= 1000.

    return df


df = pd.concat([load_data(dir) for dir in DIR_LIST], axis=0).reset_index(drop=True)

df["throughput"] = df["throughput"].fillna(0.0)
df["latency"] = df["latency"].fillna(100.0)
df["rb_rounds"] = df["rb_rounds"].fillna(100.0)
df["receive_rate"] = df["receive_rate"].fillna(0.0)

#  df = df.dropna()
#  assert df is not None

df["exp"] = df.apply(
    lambda row: "%d-%d" % (row["round_interval"], row["batched_echo_interval"]), axis=1
)
#  print(df)

payloads = list(df["payload"].unique())

group_df = df.groupby(["payload", "exp"], as_index=False).agg(
    {
        "receive_rate": ["mean", "std"],
        "rb_rounds": ["mean", "std"],
        "throughput": ["mean", "std"],
        "latency": ["mean", "std"],
    }
)
group_df.columns = [" ".join(col).strip() for col in group_df.columns.values]
group_df["round"] = group_df["exp"].apply(lambda x: int(x.split("-")[0]))
group_df["batched"] = group_df["exp"].apply(lambda x: int(x.split("-")[1]))
group_df = group_df.sort_values(by=["round", "batched", "payload"])
#  group_df = group_df[group_df['batched'] == 10]
#  group_df = group_df[group_df['round'] == 50]
print(group_df)

fig_dict = dict()

fig_title = "Reliable Broadcast Delivery Rate"
fig = px.line(
    group_df,
    x="payload",
    y="receive_rate mean",
    #  error_y='receive_rate std',
    color="exp",
    symbol="exp",
    labels={
        "exp": "Experiment",
        "payload": "Payload size (Byte)",
        "receive_rate mean": "Delivery rate (%)",
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig

fig_title = "Reliable Broadcast Number of Rounds"
fig = px.line(
    group_df,
    x="payload",
    y="rb_rounds mean",
    #  error_y='receive_rate std',
    color="exp",
    symbol="exp",
    labels={
        "exp": "Experiment",
        "payload": "Payload size (Byte)",
        "rb_rounds mean": "Number of rounds",
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig


fig_title = "Latency"
fig = px.line(
    group_df,
    x="payload",
    y="latency mean",
    #  error_y='latency std',
    color="exp",
    symbol="exp",
    labels={
        "exp": "Experiment",
        "payload": "Payload size (Byte)",
        "latency mean": "Latency (ms)",
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig


fig_title = "Throughput"
fig = px.line(
    group_df,
    x="payload",
    y="throughput mean",
    #  error_y='throughput std',
    color="exp",
    symbol="exp",
    labels={
        "exp": "Experiment",
        "payload": "Payload size (Byte)",
        "throughput mean": "Throughput (MiB/s)",
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig


for title, fig in fig_dict.items():
    fig.update_traces(marker={"size": 11})
    fig.update_layout(
        #  title = dict(text = title),
        xaxis=dict(tickmode="array", tickvals=payloads, tickformat="d")
    )
    fig.write_image(png_dir / Path(title + ".png"))
    fig.write_image(svg_dir / Path(title + ".svg"))
    fig.show()
