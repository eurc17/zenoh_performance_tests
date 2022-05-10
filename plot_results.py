#!/usr/bin/env python3

from pathlib import Path
import plotly.express as px
import pandas as pd
import json
import os


PEER_LIST = [
    "2-1",
    "2-2",
    "2-3",
    "2-4",
    "2-5",
    "2-6",
    "3-1",
    "3-2",
    "3-3",
    "3-4",
    "3-5",
    "3-6",
]



def parse_data(exp_json: Path):
    with open(exp_json) as f:
        data = json.load(f)

    peer_id = PEER_LIST[int(data["peer_id"])]
    payload = data["short_config"]["payload_size"]
    if payload > 8192:
        return None
    data = pd.DataFrame(
        [
            {
                "to": peer_id,
                "from": PEER_LIST[int(d["key_expr"].split("/")[-1])],
                "payload": payload,
                "throughput": d["throughput"],
                "latency": d["average_latency_ms"],
            }
            for d in data["result_vec"]
            if PEER_LIST[int(d["key_expr"].split("/")[-1])] != peer_id
        ]
    )
    return data


def load_data(exp_dir: str):
    df = pd.concat(
        [
            parse_data(exp_json) for exp_json
            in Path(exp_dir).glob("**/test/payload/*/exp_sub_*.json")
        ],
        axis=0
    )

    # convert throughput in MiB / sec
    #  df["throughput"] *= df["payload"] / 1024. / 1024.
    df["throughput"] *= df["payload"]

    #  # convert payload size in KiB
    #  df["payload"] /= 1024.

    # convert latency in second
    df["latency"] /= 1000.

    return df



output_dir = Path("./for_latency")
baseline_df = load_data("./baseline/exp_logs_200ms_sleep")

#  output_dir = Path("./for_throughput")
#  baseline_df = load_data("./baseline/exp_logs_05-10-16-57")

baseline_df["tag"] = "Baseline"
os.makedirs(output_dir, exist_ok=True)

rb_df = load_data("./rb/2022-05-10T17:44:29+08:00_local-ntp_round-50_echo-20/exp_logs")
rb_df["tag"] = "Reliable Broadcast"

data_df = pd.concat([baseline_df, rb_df], axis=0)

data_df["tag"] = data_df.apply(
    lambda row: row["tag"] + " " +
    ("(local)" if row["to"].split("-")[0] == row["from"].split("-")[0] else "(cross)"),
    axis=1
)

data_df = data_df.groupby(["tag", "payload"], as_index=False).agg(
    {
        "throughput": ["mean", "std"],
        "latency": ["mean", "std"]
    }
)
data_df.columns = [" ".join(col).strip() for col in data_df.columns.values]
data_df = data_df.sort_values(by=["payload", "tag"])
data_df = data_df.reset_index(drop=True)

payloads = list(data_df['payload'].unique())

data_df_by_tag = data_df.groupby("tag")
data_dict = {
    tag: data_df_by_tag \
            .get_group(tag) \
            .reset_index(drop=True) \
            .loc[:, ["payload", "latency mean", "throughput mean"]]
    for tag in data_df_by_tag.groups
}


overhead = {
    key: data_dict[f"Reliable Broadcast ({key})"]
    for key in ["local", "cross"]
}

columns = ["throughput mean", "latency mean"]
for key in overhead:
    overhead[key][columns] -= data_dict[f"Baseline ({key})"][columns]
    overhead[key]["throughput mean"] /= data_dict[f'Baseline ({key})']['throughput mean']
    overhead[key]["throughput mean"] *= -100

for key in overhead:
    overhead[key]["tag"] = "Local" if key == "local" else "Cross"
overhead_df = pd.concat(overhead.values(), axis=0)

# plot throughput overhead
fig_title = "Throughput Overhead"
fig = px.line(
    overhead_df,
    x="payload",
    y="throughput mean",
    color="tag",
    symbol="tag",
    labels={
        "tag": "Experiment",
        "payload": "Payload size (Byte)",
        "throughput mean": "Throughput Relative Difference (%)",
    },
    markers=True,
    log_x=True,
    title=fig_title,
)
fig.update_layout(xaxis = dict(tickmode = 'array', tickvals = payloads))
fig.write_image(output_dir / Path(fig_title + ".png"))
fig.write_image(output_dir / Path(fig_title + ".svg"))

# plot latency overhead
fig_title = "Latency Overhead"
fig = px.line(
    overhead_df,
    x="payload",
    y="latency mean",
    color="tag",
    symbol="tag",
    labels={
        "tag": "Experiment",
        "payload": "Payload size (Byte)",
        "latency mean": "Latency Difference (sec)",
    },
    markers=True,
    title=fig_title,
    log_x=True,
)
fig.update_layout(xaxis = dict(tickmode = 'array', tickvals = payloads))
fig.write_image(output_dir / Path(fig_title + ".png"))
fig.write_image(output_dir / Path(fig_title + ".svg"))


# plot throughput
fig_title = "Throughput"
fig = px.line(
    data_df,
    x="payload",
    y="throughput mean",
    color="tag",
    symbol="tag",
    error_y="throughput std",
    labels={
        "tag": "Experiment",
        "payload": "Payload size (Byte)",
        "throughput mean": "Throughput (MiB/sec)",
    },
    title=fig_title,
    log_x=True,
)
fig.update_layout(xaxis = dict(tickmode = 'array', tickvals = payloads))
fig.write_image(output_dir / Path(fig_title + ".png"))
fig.write_image(output_dir / Path(fig_title + ".svg"))

# plot latency
fig_title = "Latency"
fig = px.line(
    data_df,
    x="payload",
    y="latency mean",
    color="tag",
    symbol="tag",
    error_y="latency std",
    labels={
        "tag": "Experiment",
        "payload": "Payload size (Byte)",
        "latency mean": "Latency (sec)",
    },
    title=fig_title,
    log_x=True,
)
fig.update_layout(xaxis = dict(tickmode = 'array', tickvals = payloads))
fig.write_image(output_dir / Path(fig_title + ".png"))
fig.write_image(output_dir / Path(fig_title + ".svg"))
