#!/usr/bin/env python3

from pathlib import Path
import plotly.express as px
import pandas as pd
import json
import os


INPUT_DIR_FOR_THROUGHPUT = './baseline/exp_logs_05-10-16-57'
INPUT_DIR_FOR_LATENCY = './baseline/exp_logs_200ms_sleep'
OUTPUT_DIR = 'analysis_baseline'
png_dir = Path(OUTPUT_DIR) / 'png'
svg_dir = Path(OUTPUT_DIR) / 'svg'
os.makedirs(png_dir, exist_ok=True)
os.makedirs(svg_dir, exist_ok=True)


def mean(lst):
    if len(lst) == 0:
        return None
    return sum(lst) / len(lst)


def parse_data(exp_json: Path):
    with open(exp_json) as f:
        data = json.load(f)

    return dict(
        peer_id = data['short_config']['peer_id'],
        payload = data['short_config']['payload_size'],
        latency = mean([res['average_latency_ms'] for res in data['result_vec']]),
        throughput = mean([res['throughput'] for res in data['result_vec']]),
    )


def load_data(exp_dir: str):
    assert os.path.isdir(exp_dir)
    df = pd.DataFrame([
        parse_data(exp_json)
        for exp_json in Path(exp_dir).glob('**/test/payload/*/exp_sub_*.json')
    ])

    #  convert throughput in MiB / sec
    df['throughput'] *= df['payload'] / 1024. / 1024.

    #  # convert latency in second
    #  df['latency'] /= 1000.

    df = df.reset_index(drop=True)

    df = df \
        .groupby(['payload'], as_index=False) \
        .agg({
            'throughput': ['mean', 'std'],
            'latency': ['mean', 'std'],
        })
    df.columns = [' '.join(col).strip() for col in df.columns.values]
    return df


fig_dict = dict()

df = load_data(INPUT_DIR_FOR_LATENCY)
payloads = list(df['payload'].unique())
fig_title = 'Latency'
fig = px.line(
    df,
    x='payload',
    y='latency mean',
    #  error_y='latency std',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'latency mean': 'Latency (ms)',
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig


df = load_data(INPUT_DIR_FOR_THROUGHPUT)
payloads = list(df['payload'].unique())
fig_title = 'Throughput'
fig = px.line(
    df,
    x='payload',
    y='throughput mean',
    #  error_y='throughput std',
    labels={
        'exp': 'Experiment',
        'payload': 'Payload size (Byte)',
        'throughput mean': 'Throughput (MiB/s)',
    },
    markers=True,
    log_x=True,
)
fig_dict[fig_title] = fig


for title, fig in fig_dict.items():
    fig.update_layout(
        #  title = dict(text = title),
        xaxis = dict(
            tickmode = 'array',
            tickvals = payloads,
            tickformat='d'
        )
    )
    fig.write_image(png_dir / Path(title + '.png'))
    fig.write_image(svg_dir / Path(title + '.svg'))
    fig.show()
