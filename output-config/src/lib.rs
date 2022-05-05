use std::path::PathBuf;

#[derive(Debug, Parser, Serialize, Deserialize, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short = 'o', long, default_value = "./", parse(from_os_str))]
    /// The path to store the output .json file.
    pub output_dir: PathBuf,
    #[clap(short = 'p', long)]
    /// The peer ID for this process
    pub peer_id: usize,
    #[clap(short = 'a', long)]
    /// The total number of publisher peers.
    /// If pub-sub-separate flag not used, this will be the total number of peers.
    pub total_put_number: usize,
    #[clap(short = 't', long, default_value = "100")]
    /// The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).
    /// The subscriber will start receiving the messages at the same time as the publishers.
    pub round_timeout: u64,
    #[clap(short = 'i', long, default_value = "1000")]
    /// The initialization time (ms) for starting up futures.
    pub init_time: u64,
    #[clap(short = 'm', long, default_value = "1")]
    /// The number of messages each publisher peer will try to send.
    pub num_msgs_per_peer: usize,
    #[clap(short = 'n', long, default_value = "8")]
    /// The payload size (bytes) of the message.
    pub payload_size: usize,
    #[clap(long)]
    /// Create multiple zenoh runtimes on a single machine or not for each peer.
    /// It will always be set to false if pub_sub_sep is not set, since the worker will generate a new zenoh instance for each pair of pub and sub worker.
    pub multipeer_mode: bool,
    #[clap(long)]
    /// Create a zenoh runtime for a pair of pub/sub if not set.
    /// If this flag not set, the total number of peers is read from `num_put_peers`.
    pub pub_sub_separate: bool,
    #[clap(short = 'e', long)]
    /// Specifies locators for each peer to connect to (example format: tcp/x.x.x.x:7447).
    /// If you'd like to connect to several addresses, separate them with a comma (example: tcp/x.x.x.x:7447,tcp/y.y.y.y:7447)
    pub locators: Vec<Locator>,
    #[clap(short = 'r', long, default_value = "0")]
    /// Number of remote subscriber peers.
    /// Used to notify subscribers to receive messages from remote peers.
    /// Note that the num_msgs_per_peer needs to be the same on both remote and local machines
    pub remote_pub_peers: usize,
    #[clap(short = 'd', long, default_value = "0")]
    /// The time interval before starting the zenoh session, clipped by min(2000 milliseconds, 10ms * total_put_number). (Unit: ms)
    pub delay_startup: u64,
    #[clap(long, default_value = "0")]
    /// The interval between the messages published by publisher. (Unit: ms)
    pub pub_interval: u64,
    #[clap(long)]
    /// The flag that disables the publisher
    pub pub_disable: bool,
    #[clap(long)]
    /// The flag that disables the subscriber
    pub sub_disable: bool,
}

mod common;

use common::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PubTimeStatus {
    pub start_pub_worker: u128,
    pub session_start: Option<u128>,
    pub pub_sub_worker_start: Option<u128>,
    pub before_sending: u128,
    pub start_sending: u128,
    pub after_sending: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ShortConfig {
    pub peer_id: usize,
    pub total_put_number: usize,
    pub num_msgs_per_peer: usize,
    pub payload_size: usize,
    pub round_timeout: u64,
    pub init_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SubTimeStatus {
    pub process_start_sec: i64,
    pub process_start_millis: i16,
    pub start_sub_worker: u128,
    pub session_start: Option<u128>,
    pub pub_sub_worker_start: Option<u128>,
    pub after_subscribing: u128,
    pub start_receiving: u128,
    pub after_receiving: u128,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubPeerResult {
    pub key_expr: String,
    pub throughput: f64,
    pub average_latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubPeerStatistics {
    pub msg_cnt: usize,
    pub latency_vec: Vec<Duration>,
}

impl PubPeerStatistics {
    pub fn new() -> Self {
        PubPeerStatistics {
            msg_cnt: 0,
            latency_vec: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerResult {
    pub short_config: Option<ShortConfig>,
    pub peer_id: usize,
    pub receive_rate: f64,
    pub recvd_msg_num: usize,
    pub expected_msg_num: usize,
    pub result_vec: Vec<PubPeerResult>,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct TestResult {
    pub config: Cli,
    pub total_sub_returned: usize,
    pub total_receive_rate: f64,
    pub per_peer_result: Vec<PeerResult>,
}

pub fn get_msg_payload(args_payload_size: usize, peer_id: usize) -> String {
    let mut msg_payload;
    if args_payload_size == 8 {
        msg_payload = format!("{:08}", peer_id);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args_payload_size);
    } else if args_payload_size < 8 {
        warn!("Payload size cannot be less than 8 bytes, using 8 bytes for current test.");
        msg_payload = format!("{:08}", peer_id);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == 8);
    } else {
        msg_payload = format!("{:08}", peer_id);
        let additional_size = args_payload_size - 8;
        let mut postpend_string = String::from(".");
        for _ in 1..additional_size {
            postpend_string.push_str(".");
        }
        msg_payload.push_str(&postpend_string);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args_payload_size);
    }
    msg_payload
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
