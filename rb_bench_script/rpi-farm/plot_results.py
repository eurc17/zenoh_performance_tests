#!/usr/bin/env python3

from pathlib import Path
import plotly.express as px
import pandas as pd
import json
import os


PEER_LIST = [
    '2-1',
    '2-2',
    '2-3',
    '2-4',
    '2-5',
    '2-6',
    '3-1',
    '3-2',
    '3-3',
    '3-4',
    '3-5',
    '3-6',
]

input_dir = Path('./exp_logs')
output_dir = Path('./results_plotting')
os.makedirs(output_dir, exist_ok=True)



def parse_data(exp_json: Path):
    with open(exp_json) as f:
        data = json.load(f)

    peer_id = PEER_LIST[int(data['peer_id'])]
    payload = data['short_config']['payload_size']
    receive_rate = data['receive_rate']
    info = pd.DataFrame([{
        'id': peer_id,
        'payload': payload,
        'receive_rate': receive_rate
    }])
    data = pd.DataFrame([
        {
            'to': peer_id,
            'from': PEER_LIST[int(d['key_expr'].split('/')[-1])],
            'payload': payload,
            'throughput': d['throughput'],
            'latency': d['average_latency_ms']
        } for d in data['result_vec']
    ])
    return info, data


info_list = []
data_list = []
for log_dir in input_dir.glob('**/test/payload/*'):
    for exp_json in log_dir.glob('exp_sub_*.json'):
        info, data = parse_data(exp_json)
        info_list.append(info)
        data_list.append(data)

info_df = pd.concat(info_list, axis=0).sort_values(by=['payload', 'id'])
data_df = pd.concat(data_list, axis=0)

# # Unusable
#  fig = px.line(
#      info_df,
#      x='payload',
#      y='receive_rate',
#      color='id'
#  )
#  fig.show()



# convert throughput in MiB / sec
data_df['throughput'] *= data_df['payload'] / 1024 / 1024

# convert payload size in KiB
data_df['payload'] /= 1024

# convert latency in second
data_df['latency'] /= 1000

data_df['local'] = data_df.apply(lambda row: row['to'].split('-')[0] == row['from'].split('-')[0], axis=1)

for is_local in [True, False]:
    df = data_df[data_df['local'] == is_local]
    df = df.groupby(['to', 'payload'], as_index=False).agg({'throughput': ['mean', 'std'], 'latency': ['mean', 'std']})
    df.columns = [' '.join(col).strip() for col in df.columns.values]
    df = df.sort_values(by=['payload', 'to'])

    # plot throughput
    fig_title = 'Throughput (%s)' % ('local' if is_local else 'cross')
    fig = px.line(
        df,
        x='payload',
        y='throughput mean',
        color='to',
        error_y='throughput std',
        labels={
            'to': 'Peer',
            'payload': 'Payload size (KiB)',
            'throughput mean': 'Throughput (MiB/sec)',
        },
        title=fig_title,
        log_x=True,
    )
    fig.write_image(output_dir / Path(fig_title + '.png'))


    # plot latency
    fig_title = 'Latency (%s)' % ('local' if is_local else 'cross')
    fig = px.line(
        df,
        x='payload',
        y='latency mean',
        color='to',
        error_y='latency std',
        labels={
            'to': 'Peer',
            'payload': 'Payload size (KiB)',
            'latency mean': 'Latency (sec)',
        },
        title=fig_title,
        log_x=True,
    )
    fig.write_image(output_dir / Path(fig_title + '.png'))


print(f'Plotting have been outputted to {output_dir}.')
