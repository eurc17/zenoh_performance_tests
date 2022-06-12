# Notes on running experiments on RPis with zenoh-performance-tests

## Instructions

1. Clone zenoh-performance-tests
2. Switch to branch thoughput-latency
3. Enter directory exp_script
4. Run make sync_zpt_binary to build and sent the binary & repository
   to the RPis
5. Run make execute_exp_full_cross_net_no_rust_log to run experiments
   on RPis, the output of the terminals will be logged to files, but
   the RUST_LOG will not be set. The results along with the logs will
   be automatically cloned to exp_script/exp_logs once the experiment
   finishes
6. (Alternative experiment, sets RUST_LOG=debug) Run make
   execute_exp_full_cross_net to run experiments on RPis, the output
   of the terminals will be logged to files with RUST_LOG set to
   debug. The results along with the logs will be automatically cloned
   to exp_script/exp_logs once the experiment finishes
7. To clean up results and logs in RPis, run make remove_tests_rpi

## Additional notes

- There's NO need to manually start the zenohd on router. The
  experiment command will automatically start and stop the zenoh
  router.
- Please do not cancel the experiment once it starts. Wait until it
  terminates. The program has a timeout mechanism.
- The payload sizes of the messages are read from
  scripts/exp_payload_list.txt , the more payload configurations
  added, the more experiments will be run.
