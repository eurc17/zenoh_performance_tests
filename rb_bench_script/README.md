# Reliable Broadcast Benchmark Guide

## Build and Deploy

Execute `run.sh`. It compiles the binary for Raspberry Pi, distributes
the binaries to client Pis and launches zenohd on router Pi, and runs
the tests and download the logs in one step.

```sh
./run.sh
```

## Output files


The files are copied to `/home/pi/rb-exp` on RPi devices. The path is
configurable.

The output logs is located at the directory below. The hierarchy
follows the Vincent's convention.

```
/home/pi/rb-exp/test/payload/
```

The program saves stdout/stderr to:

```
/home/pi/rb-exp/rb_2022-05-10-07:14:01_4096.stdout
/home/pi/rb-exp/rb_2022-05-10-07:14:01_4096.stderr
```

## Configuration

### Program arguments 

`conifig/command_args.template` defines program arguments passed to
the binary to be executed.

```
--total-put-number 1 \
--remote-pub-peers 11 \
--locators tcp/192.168.1.121:7447 \
--init-time 5000 \
...
```

The most important parameters for Reliable Broadcast (RB) are the
follows. They depend on the payload size.

- The min latency is `(1 + extra-rounds) * round-interval` milliseconds, while 
- the RB runs up to `(max-rounds + extra-round) * round-interval` milliseconds.
- `echo-interval` < `round-interval`

```
--round-interval 40 \
--echo-interval 20 \
--max-rounds 3 \
--extra-rounds 1 \
```

The suggested values for respective payload sizes are shown in the
table to reach 100% message delivery rate.

| psize | echo interval | round interval |
|-------|---------------|----------------|
| <4096 | 20            | 40             |
| 8192  | 20            | 50             |
| 16384 | 30            | 100            |
| 65500 | 100           | >1000 (fail)   |


### Device addresses

`conifig/rpi_addrs.txt` and `conifig/router_addrs.txt` enumerates the
SSH network address and peer ID for each Raspberry device.


### Test Environment Variables

`step/00_config.sh` defines parameters for the testing environment,
such as the directory path to store output files on Raspberry Pis.

The `payload_sizes` is a space-delimited list of payload sizes in
bytes. It can be loaded from a file or be manually specified.

```sh
# load from file
payload_sizes=$(cat "$repo_dir/scripts/exp_payload_list.txt")

# manual specification
payload_sizes="128 4096"
```

The `remote_rust_log` defines the `RUST_LOG` environment variable on
Raspberry device. It is disabled by default.

```
# disable by default
remote_rust_log=""

# suggested for debugging purpose
remote_rust_log="reliable_broadcast=debug,reliable_broadcast_benchmark=debug"
```

## Debugging


Set the `remote_rust_log` value to let programs to dump debug
messages.

To inspect the most recent log for payload size 128 a RPi device, the
snipplet does the trick.

```sh
ssh -p 44001 pi@192.168.1.102 \
  'cd ~/rb-exp && cat $(ls rb_*_128.stderr | sort -r | head -n1)'
```

To count the occurrences of acceptances and rejections for broadcasted messages,

```sh
ssh -p 44001 pi@192.168.1.102 \
  'cd ~/rb-exp && cat $(ls rb_*_128.stderr | sort -r | head -n1)' | \
  grep -e accepts -e rejects
```

## Troubleshooting

If you encounter issues, try to use the following scripts to reset the
environment on RPi.

- `./kill_remote.sh` kills zenohd and orphan process on RPis.
- `./clean_remote.sh` files on RPi devices, including log files.
