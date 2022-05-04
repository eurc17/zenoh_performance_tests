use crate::Cli;

use super::common::*;

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
