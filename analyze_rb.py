#!/usr/bin/env python3

from pathlib import Path
import plotly.express as px
import pandas as pd
import json
import os



DIR_LIST = [
    './rb/2022-05-11 16:52:16+08:00_round-50_echo-1',
    #  './rb/2022-05-11 16:36:54+08:00_round-50_echo-20',
    './rb/2022-05-11 16:57:56+08:00_round-100_echo-40',
    './rb/2022-05-11 21:42:43+08:00_round-50_echo-20_1',
]
OUTPUT_DIR = 'analysis_rb'
png_dir = Path(OUTPUT_DIR) / 'png'
svg_dir = Path(OUTPUT_DIR) / 'svg'
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
        peer_id = data['short_config']['peer_id'],
        payload = data['short_config']['payload_size'],
        round_interval = round_interval,
        batched_echo_interval = batched_echo_interval,
        receive_rate = data['receive_rate'] * 100,
        rb_rounds = data['average_rb_rounds'],
        latency = mean([res['average_latency_ms'] for res in data['result_vec']]),
        throughput = mean([res['throughput'] for res in data['result_vec']]),
    )


def load_data(exp_dir: str):
    assert os.path.isdir(exp_dir)
    round_interval = int(exp_dir.split('round-')[-1].split('_')[0])
    batched_echo_interval = int(exp_dir.split('echo-')[-1].split('_')[0])
    df = pd.DataFrame([
        parse_data(
            exp_json,
            round_interval,
            batched_echo_interval
        )
        for exp_json in Path(exp_dir).glob('**/test/payload/*/exp_sub_*.json')
    ])

    #  convert throughput in MiB / sec
    df['throughput'] *= df['payload'] / 1024. / 1024.
    #  df['throughput'] *= df['payload']

    #  # convert latency in second
    #  df['latency'] /= 1000.

    return df



df = pd.concat([load_data(dir) for dir in DIR_LIST], axis=0).reset_index(drop=True)
#  df = df.fillna(10.)
df = df.dropna()
assert df is not None
df['exp'] = df.apply(lambda row: '%d-%d' % (row['round_interval'], row['batched_echo_interval']), axis=1)
print(df)

payloads = list(df['payload'].unique())

group_df = df \
    .groupby(['payload', 'exp'], as_index=False) \
    .agg({
        'receive_rate': ['mean', 'std'],
        'rb_rounds': ['mean', 'std'],
        'throughput': ['mean', 'std'],
        'latency': ['mean', 'std'],
    })
group_df.columns = [' '.join(col).strip() for col in group_df.columns.values]
print(group_df)

fig_dict = dict()

fig_title = 'Reliable Broadcast Receive Rate'
fig = px.line(
    group_df,
    x='payload',
    y='receive_rate mean',
    #  error_y='receive_rate std',
    color='exp',
    symbol='exp',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'receive_rate mean': 'Receive rate (%)',
    },
    markers=True,
    title=fig_title,
    log_x=True,
)
fig_dict[fig_title] = fig

fig_title = 'Reliable Broadcast Number of Rounds'
fig = px.line(
    group_df,
    x='payload',
    y='rb_rounds mean',
    #  error_y='receive_rate std',
    color='exp',
    symbol='exp',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'rb_rounds mean': 'Number of rounds',
    },
    markers=True,
    title=fig_title,
    log_x=True,
)
fig_dict[fig_title] = fig


fig_title = 'Latency'
fig = px.line(
    group_df,
    x='payload',
    y='latency mean',
    #  error_y='latency std',
    color='exp',
    symbol='exp',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'latency mean': 'Latency (ms)',
    },
    markers=True,
    title=fig_title,
    log_x=True,
)
fig.update_layout(yaxis = dict(dtick = 200))
fig_dict[fig_title] = fig


fig_title = 'Throughput'
fig = px.line(
    group_df,
    x='payload',
    y='throughput mean',
    #  error_y='throughput std',
    color='exp',
    symbol='exp',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'throughput mean': 'Throughput (MiB/s)',
    },
    markers=True,
    title=fig_title,
    log_x=True,
)
fig_dict[fig_title] = fig


for title, fig in fig_dict.items():
    fig.update_layout(xaxis = dict(
        tickmode = 'array',
        tickvals = payloads,
        tickformat='d'
    ))
    fig.write_image(png_dir / Path(title + '.png'))
    fig.write_image(svg_dir / Path(title + '.svg'))
    fig.show()
