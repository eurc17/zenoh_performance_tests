import subprocess
import os
import glob
import argparse
import matplotlib.pyplot as plt
import json


def main(args):
    print(args)
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
    payload_list = []
    mem_list = []
    cpu_list = []
    recv_rate_list = []
    for json_file in sorted(glob.glob(args.input_dir + "/*.json")):
        with open(json_file) as json_opened_file:
            data = json.load(json_opened_file)
            recv_rate = float(data["total_receive_rate"])
            recv_rate_list.append(recv_rate)

        file = (
            args.input_dir
            + "/log-"
            + os.path.basename(json_file).split(".json")[0]
            + ".txt"
        )
        print(file)
        payload_size = int(os.path.basename(file).split("-")[4])

        result = subprocess.run(
            [
                "./target/release/usage-parser",
                "-i",
                file,
            ],
            capture_output=True,
            text=True,
        )
        # Real Memory Usage
        mem_usage = float(result.stderr.split("\n")[1].split("= ")[-1].split(" MB")[0])
        # CPU Usage
        cpu_usage = float(result.stderr.split("\n")[0].split("= ")[-1].split("%")[0])
        payload_list.append(payload_size)
        mem_list.append(mem_usage)
        cpu_list.append(cpu_usage)
        print(payload_size, mem_usage, cpu_usage)
    fig, ax1 = plt.subplots()

    color = "tab:red"
    ax1.set_xlabel("payload size (bytes)")
    ax1.set_ylabel("peak cpu usage (%)", color=color)
    ax1.plot(payload_list, cpu_list, color=color)
    ax1.tick_params(axis="y", labelcolor=color)

    ax2 = ax1.twinx()  # instantiate a second axes that shares the same x-axis

    color = "tab:blue"
    ax2.set_ylabel("peak memory usage (MB)", color=color)
    ax2.plot(payload_list, mem_list, color=color)
    ax2.tick_params(axis="y", labelcolor=color)

    fig.tight_layout()  # otherwise the right y-label is slightly clipped
    plt.savefig(args.output_dir + "/resource.png")
    # plt.show()
    plt.clf()
    plt.plot(payload_list, recv_rate_list, label="msg recv rate")
    plt.savefig(args.output_dir + "/recv_rate.png")
    plt.show()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run multiple zenoh-performance-test with increasing peer numbers"
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
