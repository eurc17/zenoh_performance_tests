mod common;

use common::*;
use std::{iter, path::PathBuf};

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
    /// Number of remote publisher peers.
    /// Used to notify subscribers to receive messages from remote peers.
    /// Note that the num_msgs_per_peer needs to be the same on both remote and local machines
    pub remote_pub_peers: usize,
    #[clap(short = 'd', long, default_value = "0")]
    /// The time interval before starting the zenoh session, clipped by min(2000 milliseconds, 10ms * total_put_number). (Unit: ms)
    pub delay_startup: u64,
    #[clap(long, default_value = "1")]
    /// The interval between the messages published by publisher. (Unit: ms)
    pub pub_interval: u64,
    #[clap(long)]
    /// The flag that disables the publisher
    pub pub_disable: bool,
    #[clap(long)]
    /// The flag that disables the subscriber
    pub sub_disable: bool,

    // params for Reliable Broadcast
    #[clap(long)]
    /// Subscription mode for Zenoh
    pub sub_mode: Option<SubMode>,
    #[clap(long)]
    /// Reliability QoS for Zenoh
    pub reliability: Option<Reliability>,
    #[clap(long)]
    /// Congestion control QoS for Zenoh
    pub congestion_control: Option<CongestionControl>,
    #[clap(long)]
    /// The waiting time to publish batched echo in milliseconds for RB
    pub echo_interval: Option<u64>,
    #[clap(long)]
    /// The waiting time per round for RB
    pub round_interval: Option<u64>,
    #[clap(long)]
    /// Max # of RB rounds
    pub max_rounds: Option<usize>,
    #[clap(long)]
    /// # of extra rounds for RB
    pub extra_rounds: Option<usize>,
}

impl Cli {
    pub fn test_timeout(&self) -> Duration {
        Duration::from_millis(self.round_timeout)
    }

    pub fn round_interval(&self) -> Duration {
        Duration::from_millis(self.round_interval.unwrap())
    }

    pub fn echo_interval(&self) -> Duration {
        Duration::from_millis(self.echo_interval.unwrap())
    }

    pub fn publish_interval(&self) -> Duration {
        (self.round_interval() * self.max_rounds.unwrap() as u32) + Duration::from_millis(50)
    }

    pub fn start_time_from_now(&self) -> Instant {
        Instant::now() + Duration::from_millis(self.init_time)
    }

    pub fn total_msg_num(&self) -> usize {
        (self.total_put_number + self.remote_pub_peers) * self.num_msgs_per_peer
    }
}

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

impl<'a> From<&'a Cli> for ShortConfig {
    fn from(cli: &'a Cli) -> Self {
        Self {
            peer_id: cli.peer_id,
            total_put_number: cli.total_put_number,
            num_msgs_per_peer: cli.num_msgs_per_peer,
            payload_size: cli.payload_size,
            round_timeout: cli.round_timeout,
            init_time: cli.init_time,
        }
    }
}

impl From<Cli> for ShortConfig {
    fn from(cli: Cli) -> Self {
        (&cli).into()
    }
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

impl Default for PubPeerStatistics {
    fn default() -> Self {
        Self::new()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CongestionControl {
    Block,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SubMode {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Reliability {
    BestEffort,
    Reliable,
}

impl From<CongestionControl> for zenoh::publication::CongestionControl {
    fn from(from: CongestionControl) -> Self {
        type F = CongestionControl;

        match from {
            F::Block => Self::Block,
            F::Drop => Self::Drop,
        }
    }
}

impl From<SubMode> for zenoh::subscriber::SubMode {
    fn from(from: SubMode) -> Self {
        type F = SubMode;

        match from {
            F::Push => Self::Push,
            F::Pull => Self::Pull,
        }
    }
}

impl From<Reliability> for zenoh::subscriber::Reliability {
    fn from(from: Reliability) -> Self {
        type F = Reliability;

        match from {
            F::BestEffort => Self::BestEffort,
            F::Reliable => Self::Reliable,
        }
    }
}

pub fn get_msg_payload(len: usize, peer_id: u64) -> Arc<[u8]> {
    iter::repeat(peer_id.to_le_bytes())
        .flatten()
        .take(len)
        .collect()
}

pub fn peer_id_from_payload(payload: &[u8]) -> u64 {
    u64::from_le_bytes(payload[0..8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
