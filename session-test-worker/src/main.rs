mod common;

use crate::common::*;
use std::path::PathBuf;
use std::{io::Write, str::FromStr};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short = 'o', long, default_value = "./", parse(from_os_str))]
    /// The path to store the output .json file.
    output_dir: PathBuf,
    #[clap(short = 'p', long)]
    /// The peer ID for this process
    peer_id: usize,
    #[clap(short = 'a', long)]
    /// The total number of publisher peers.
    /// If pub-sub-separate flag not used, this will be the total number of peers.
    num_put_peer: usize,
    #[clap(short = 't', long, default_value = "100")]
    /// The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).
    /// The subscriber will start receiving the messages at the same time as the publishers.
    round_timeout: u64,
    #[clap(short = 'i', long, default_value = "1000")]
    /// The initialization time (ms) for starting up futures.
    init_time: u64,
    #[clap(short = 'm', long, default_value = "1")]
    /// The number of messages each publisher peer will try to send.
    num_msgs_per_peer: usize,
    #[clap(short = 'n', long, default_value = "8")]
    /// The payload size (bytes) of the message.
    payload_size: usize,
    #[clap(long)]
    /// The number of tasks to spawn for dealing with futures related to publisher peers.
    pub_cpu_num: Option<usize>,
    #[clap(long)]
    /// The number of tasks to spawn for dealing with futures related to subscriber peers.
    sub_cpu_num: Option<usize>,
    #[clap(long)]
    /// Create multiple zenoh runtimes on a single machine or not for each peer.
    /// It will always be set to false if pub_sub_sep is not set, since the worker will generate a new zenoh instance for each pair of pub and sub worker.
    multipeer_mode: bool,
    #[clap(long)]
    /// Create a zenoh runtime for a pair of pub/sub if not set.
    /// If this flag not set, the total number of peers is read from `num_put_peers`.
    pub_sub_separate: bool,
    #[clap(short = 'e', long)]
    /// Specifies locators for each peer to connect to (example format: tcp/x.x.x.x:7447).
    /// If you'd like to connect to several addresses, separate them with a comma (example: tcp/x.x.x.x:7447,tcp/y.y.y.y:7447)
    locators: Option<String>,
    #[clap(short = 'r', long, default_value = "0")]
    /// Number of remote subscriber peers.
    /// Used to notify subscribers to receive messages from remote peers.
    /// Note that the num_msgs_per_peer needs to be the same on both remote and local machines
    remote_pub_peers: usize,
    #[clap(short = 'd', long, default_value = "0")]
    delay_startup: u64,
    #[clap(short = 's', long, default_value = "0.2")]
    scout_delay: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeStatus {
    pub session_start: Option<u128>,
    pub pub_sub_worker_start: Option<u128>,
    pub list_start_timestamp: Vec<u128>,
    pub list_sess_start_timestamp: Vec<u128>,
    pub list_timestamp_peer_num: Vec<usize>,
    pub list_timestamp_res: Vec<Vec<String>>,
    pub session_id: Option<String>,
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

pub async fn pub_and_sub_worker(
    _start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    num_msgs_per_peer: usize,
    _msg_payload: String,
    _total_msg_num: usize,
    locators: Option<String>,
    _output_dir: PathBuf,
    total_put_number: usize,
    payload_size: usize,
    args: Cli,
    start: Instant,
) -> anyhow::Result<()> {
    let pub_sub_worker_start = Some(Instant::now());
    let mut config = config::default();
    if let Some(locators) = locators.clone() {
        let endpoints = locators
            .split(",")
            .filter(|str| *str != "")
            .map(|locator| EndPoint::from(Locator::from_str(locator).unwrap()))
            .collect::<Vec<_>>();
        let listerner_config = ListenConfig { endpoints };
        config.set_listen(listerner_config).unwrap();
    }
    config.scouting.set_delay(Some(args.scout_delay)).unwrap();
    let zenoh = Arc::new(zenoh::open(config).await.unwrap());
    let session_start_time = Some(Instant::now());
    let mut list_start_timestamp: Vec<u128> = vec![];
    let mut list_sess_start_timestamp: Vec<u128> = vec![];
    let mut list_timestamp_peer_num: Vec<usize> = vec![];
    let mut list_timestamp_res: Vec<Vec<String>> = vec![];
    let mut session_id: Option<String> = None;

    while Instant::now() < timeout || session_id.is_none() {
        // Todo: Sleep and get duration & peer info
        let _sleep_end_time = Instant::now();
        let session_info = zenoh.info().await;
        let after_session_info = Instant::now();
        if session_id.is_none() {
            session_id = Some(session_info.get(&0).unwrap().clone());
        }
        list_start_timestamp.push((after_session_info - start).as_millis());
        list_sess_start_timestamp
            .push((after_session_info - session_start_time.unwrap()).as_millis());
        let curr_peer_num = session_info
            .get(&1)
            .unwrap()
            .split(",")
            .map(|s| s.to_string())
            .filter(|s| s != "")
            .collect::<Vec<String>>();
        list_timestamp_peer_num.push(curr_peer_num.len());
        list_timestamp_res.push(curr_peer_num);
        // println!(
        //     "Process time = {} ms",
        //     (Instant::now() - after_session_info).as_millis()
        // );

        async_std::task::sleep(Duration::from_millis(100)).await;
    }

    let zenoh = Arc::try_unwrap(zenoh).map_err(|_| ()).unwrap();
    zenoh.close().await.unwrap();

    let file_path = args.output_dir.join(format!(
        "Session_{}_info_{}-{}-{}-{}-{}-{}.json",
        peer_id,
        total_put_number,
        total_put_number,
        num_msgs_per_peer,
        payload_size,
        args.round_timeout,
        args.init_time
    ));
    let session_start = Some((session_start_time.unwrap() - start).as_millis());
    let pus_sub_work_start_dur = Some((pub_sub_worker_start.unwrap() - start).as_millis());
    let test_result = TimeStatus {
        session_start,
        pub_sub_worker_start: pus_sub_work_start_dur,
        list_start_timestamp,
        list_sess_start_timestamp,
        list_timestamp_peer_num,
        list_timestamp_res,
        session_id,
    };

    let mut file = std::fs::File::create(file_path).unwrap();
    writeln!(
        &mut file,
        "{}",
        serde_json::to_string_pretty(&test_result).unwrap()
    )
    .unwrap();

    Ok(())
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    // Get & parse arguments
    let args = Cli::parse();
    let default_wait_time = (10 * args.num_put_peer as u64).max(2000);

    async_std::task::sleep(Duration::from_millis(
        default_wait_time - args.delay_startup,
    ))
    .await;

    // Parameters
    let start = Instant::now();
    println!("Peer {}, start = {:?}", args.peer_id, start);
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);
    let total_put_number = args.num_put_peer;
    let total_cpu_num = num_cpus::get();
    let available_cpu_num = (total_cpu_num - 2).max(1);
    let _per_peer_num = total_put_number / available_cpu_num;

    let pub_sub_futs = pub_and_sub_worker(
        start_until,
        timeout,
        args.peer_id,
        args.num_msgs_per_peer,
        get_msg_payload(args.payload_size, args.peer_id),
        (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
        args.locators.clone(),
        args.output_dir.clone(),
        total_put_number,
        args.payload_size,
        args.clone(),
        start,
    );

    let _result = futures::join!(pub_sub_futs);
}
