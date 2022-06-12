from functools import cmp_to_key
import os
import glob
import argparse
import json
from numpy import append
import plotly.offline as pyo
import plotly.graph_objs as go
from plotly.subplots import make_subplots
from dataclasses import dataclass

from pyrsistent import get_in


@dataclass
class ElapsedTime:
    hours: int
    minutes: int
    seconds: float

    def to_seconds(self) -> float:
        return self.hours * 3600.0 + self.minutes * 60.0 + self.seconds


@dataclass
class TimeLog:
    user_time_sec: float
    sys_time_sec: float
    cpu_percent: int
    elapsed_time: ElapsedTime
    avg_shr_txt_size_KiB: int
    avg_unshr_txt_size_KiB: int
    avg_stack_size_KiB: int
    avg_total_size_KiB: int
    max_res_set_size_KiB: int
    avg_res_set_size_KiB: int
    major_page_faults: int
    minor_page_faults: int
    vol_context_switch: int
    invol_context_switch: int
    swaps: int
    fs_inputs: int
    fs_outputs: int
    socket_msg_sent: int
    socket_msg_recv: int
    sig_delivered: int
    page_size_B: int
    exit_status: int


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


def get_float_from_str(string: str) -> float:
    value_str = string.split(":")[1].strip()
    return float(value_str)


def get_int_from_str(string: str) -> int:
    value_str = string.split(":")[1].strip().split("%")[0]
    return int(value_str)


def get_elapsed_time_from_str(string: str) -> ElapsedTime:
    elapsed_str = string.split("):")[1].strip()
    time_list = elapsed_str.split(":")
    if len(time_list) == 3:
        return ElapsedTime(int(time_list[0]), int(time_list[1]), float(time_list[2]))
    elif len(time_list) == 2:
        return ElapsedTime(0, int(time_list[0]), float(time_list[1]))
    else:
        print("Elapsed time parsing error: Time format is not (h:mm:ss) or (m:ss)")


def get_data_from_file(file_path: str) -> TimeLog:
    with open(file_path, "r") as open_file:
        line = open_file.readline()
        while line != "":
            if "User time (seconds)" in line:
                user_time_sec = get_float_from_str(line)
            if "System time (seconds)" in line:
                sys_time_sec = get_float_from_str(line)
            if "Percent of CPU this job got" in line:
                cpu_percent = get_int_from_str(line)
            if "Elapsed (wall clock) time (h:mm:ss or m:ss)" in line:
                elapsed_time = get_elapsed_time_from_str(line)
            if "Average shared text size" in line:
                avg_shr_txt_size_KiB = get_int_from_str(line)
            if "Average unshared data size" in line:
                avg_unshr_txt_size_KiB = get_int_from_str(line)
            if "Average stack size" in line:
                avg_stack_size_KiB = get_int_from_str(line)
            if "Average total size" in line:
                avg_total_size_KiB = get_int_from_str(line)
            if "Maximum resident set size" in line:
                max_res_set_size_KiB = get_int_from_str(line)
            if "Average resident set size" in line:
                avg_res_set_size_KiB = get_int_from_str(line)
            if "Major (requiring I/O) page faults" in line:
                major_page_faults = get_int_from_str(line)
            if "Minor (reclaiming a frame) page faults" in line:
                minor_page_faults = get_int_from_str(line)
            if "Voluntary context switches" in line:
                vol_context_switch = get_int_from_str(line)
            if "Involuntary context switches" in line:
                invol_context_switch = get_int_from_str(line)
            if "Swaps" in line:
                swaps = get_int_from_str(line)
            if "File system inputs" in line:
                fs_inputs = get_int_from_str(line)
            if "File system outputs" in line:
                fs_outputs = get_int_from_str(line)
            if "Socket messages sent" in line:
                socket_msg_sent = get_int_from_str(line)
            if "Socket messages received" in line:
                socket_msg_recv = get_int_from_str(line)
            if "Signals delivered" in line:
                sig_delivered = get_int_from_str(line)
            if "Page size (bytes)" in line:
                page_size_B = get_int_from_str(line)
            if "Exit status" in line:
                exit_status = get_int_from_str(line)
        ret = TimeLog(
            user_time_sec,
            sys_time_sec,
            cpu_percent,
            elapsed_time,
            avg_shr_txt_size_KiB,
            avg_unshr_txt_size_KiB,
            avg_stack_size_KiB,
            avg_total_size_KiB,
            max_res_set_size_KiB,
            avg_res_set_size_KiB,
            major_page_faults,
            minor_page_faults,
            vol_context_switch,
            invol_context_switch,
            swaps,
            fs_inputs,
            fs_outputs,
            socket_msg_sent,
            socket_msg_recv,
            sig_delivered,
            page_size_B,
            exit_status,
        )
    return ret


def plot_usage(exp_dict, output_dir, dist=0.15):
    for exp_key in exp_dict:
        fig = make_subplots(specs=[[{"secondary_y": True}]])
        # fig = go.Figure()
        total_cpu_usage = []
        total_mem_usage = []
        time_diff_sum = 0
        cnt = 0
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
                if i != 0:
                    cnt += 1
                    time_diff_sum += data_peer[i][0] - data_peer[i - 1][0]
        avg_time_diff = time_diff_sum / cnt

        # Create figure with secondary y-axis

        # Add traces
        fig.add_trace(
            go.Scatter(
                x=[i * avg_time_diff for i in range(0, len(total_cpu_usage))],
                y=total_cpu_usage,
                name="CPU Usage",
            ),
            secondary_y=False,
        )

        fig.add_trace(
            go.Scatter(
                x=[i * avg_time_diff for i in range(0, len(total_mem_usage))],
                y=total_mem_usage,
                name="Mem Usage",
            ),
            secondary_y=True,
        )

        # # Update layout
        fig.update_layout(
            title=exp_key,
            xaxis_title="Time (s)",
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
