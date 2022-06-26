mod common;
mod utils;
mod workers;
use clap::Parser;
use common::*;
use std::path::PathBuf;
use utils::*;
use workers::*;

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short = 'o', long, default_value = "./", parse(from_os_str))]
    /// The path to store the output .json file.
    output_dir: PathBuf,
    #[clap(short = 'p', long, default_value = "1")]
    /// The total number of publisher peers.
    /// If pub-sub-separate flag not used, this will be the total number of peers.
    num_put_peer: usize,
    #[clap(short = 's', long, default_value = "1")]
    /// The total number of subscriber peers.
    num_sub_peer: usize,
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
    /// Create multiple zenoh runtimes on a single machine or not for each peer. \
    /// It will always be set to false if pub_sub_sep is not set, since the worker will generate a new zenoh instance for each pair of pub and sub worker.
    multipeer_mode: bool,
    #[clap(long)]
    /// Create a zenoh runtime for a pair of pub/sub if not set.
    /// If this flag not set, the total number of peers is read from `num_put_peers`.
    pub_sub_separate: bool,
    #[clap(short = 'e', long, value_delimiter = ',')]
    /// Specifies locators for each peer to connect to (example format: tcp/x.x.x.x:7447).
    /// If you'd like to connect to several addresses, separate them with a comma (example: tcp/x.x.x.x:7447,tcp/y.y.y.y:7447)
    locators: Vec<Locator>,
    #[clap(short = 'a', long, default_value = "0")]
    /// Number of remote subscriber peers.
    /// Used to notify subscribers to receive messages from remote peers.
    /// Note that the num_msgs_per_peer needs to be the same on both remote and local machines
    remote_pub_peers: usize,
}
#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let args = Cli::parse();
    dbg!(&args);
    println!("# of CPU cores = {}", num_cpus::get());
    if args.pub_sub_separate {
        test_worker_1(args).await;
    } else {
        test_pub_and_sub_worker(args).await;
    }
}

async fn test_pub_and_sub_worker(args: Cli) {
    let (tx, rx) = flume::unbounded::<(usize, Vec<Sample>)>();

    let start = Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);
    let total_sub_number = args.num_put_peer;
    let total_put_number = args.num_put_peer;
    let total_cpu_num = num_cpus::get();
    let available_cpu_num = (total_cpu_num - 2).max(1);
    let per_peer_num = total_put_number / available_cpu_num;

    if total_put_number < available_cpu_num {
        let pub_sub_futs = (0..total_put_number)
            .into_par_iter()
            .map(|peer_index| {
                async_std::task::spawn(pub_and_sub_worker(
                    start_until,
                    timeout,
                    peer_index,
                    args.num_msgs_per_peer,
                    get_msg_payload(args.payload_size, peer_index),
                    tx.clone(),
                    (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
                    args.locators.clone(),
                    args.output_dir.clone(),
                    total_put_number,
                    args.payload_size,
                    args.clone(),
                    start,
                ))
            })
            .collect::<Vec<_>>();
        let all_fut = futures::future::join_all(pub_sub_futs);

        let demo_fut = demonstration_worker(
            rx,
            total_put_number,
            total_sub_number,
            args.num_msgs_per_peer,
            args.remote_pub_peers,
            args.payload_size,
            args.round_timeout,
            args.clone(),
        );

        drop(tx);

        futures::join!(all_fut, demo_fut);
    } else {
        let mut pub_sub_futs = (0..available_cpu_num)
            .into_par_iter()
            .map(|core_idx| {
                let pub_sub_futures = (0..per_peer_num)
                    .into_par_iter()
                    .map(|peer_index| {
                        pub_and_sub_worker(
                            start_until,
                            timeout,
                            peer_index + core_idx * per_peer_num,
                            args.num_msgs_per_peer,
                            get_msg_payload(args.payload_size, peer_index),
                            tx.clone(),
                            (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
                            args.locators.clone(),
                            args.output_dir.clone(),
                            total_put_number,
                            args.payload_size,
                            args.clone(),
                            start,
                        )
                    })
                    .collect::<Vec<_>>();
                async_std::task::spawn(futures::future::join_all(pub_sub_futures))
            })
            .collect::<Vec<_>>();

        let remaining_pub_sub = total_put_number % available_cpu_num;
        let remaining_pub_sub_fut = (total_put_number - remaining_pub_sub..total_put_number)
            .into_par_iter()
            .map(|peer_index| {
                pub_and_sub_worker(
                    start_until,
                    timeout,
                    peer_index,
                    args.num_msgs_per_peer,
                    get_msg_payload(args.payload_size, peer_index),
                    tx.clone(),
                    (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
                    args.locators.clone(),
                    args.output_dir.clone(),
                    total_put_number,
                    args.payload_size,
                    args.clone(),
                    start,
                )
            })
            .collect::<Vec<_>>();

        let remaining_pub_sub_fut =
            async_std::task::spawn(futures::future::join_all(remaining_pub_sub_fut));
        pub_sub_futs.push(remaining_pub_sub_fut);

        let all_fut = futures::future::join_all(pub_sub_futs);

        let demo_fut = demonstration_worker(
            rx,
            total_put_number,
            total_sub_number,
            args.num_msgs_per_peer,
            args.remote_pub_peers,
            args.payload_size,
            args.round_timeout,
            args.clone(),
        );

        drop(tx);

        futures::join!(all_fut, demo_fut);
    }
}

async fn test_worker_1(args: Cli) {
    let (tx, rx) = flume::unbounded::<(usize, Vec<Sample>)>();
    let mut config = config::default();

    let endpoints = args
        .locators
        .clone()
        .into_iter()
        .map(|locator| EndPoint::from(locator))
        .collect::<Vec<_>>();
    let listerner_config = ListenConfig { endpoints };
    config.set_listen(listerner_config).unwrap();

    let zenoh = Arc::new(zenoh::open(config).await.unwrap());

    let start = Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);
    let total_sub_number = args.num_sub_peer;
    let total_put_number = args.num_put_peer;
    let total_cpu_num = num_cpus::get();
    let available_cpu_num = (total_cpu_num - 2).max(1);
    if args.sub_cpu_num.is_some() && args.pub_cpu_num.is_some() {
        if args.sub_cpu_num.unwrap() + args.pub_cpu_num.unwrap() > available_cpu_num {
            warn!("Spawning more than available cpu cores tasks for pub/sub.");
        }
    }

    // Old subscriber
    // let sub_handle_vec = (0..total_sub_number)
    //     .into_par_iter()
    //     .map(|peer_id: usize| {
    //         let sub_handle =
    //             subscribe_worker(zenoh.clone(), start_until, timeout, peer_id, tx.clone());
    //         sub_handle
    //     })
    //     .collect::<Vec<_>>();

    // new subscriber
    let sub_cpu_num;
    if let Some(args_sub_cpu_num) = args.sub_cpu_num {
        sub_cpu_num = args_sub_cpu_num;
        if args_sub_cpu_num > (available_cpu_num + 1) / 2 {
            warn!("Spawning more than half of available cpu cores tasks for subscribers.");
        }
    } else {
        sub_cpu_num = (available_cpu_num + 1) / 2;
    }
    println!("Will spawn up to {} tasks for subscribers", sub_cpu_num + 1);

    let sub_per_peer_num = total_sub_number / sub_cpu_num;
    let mut sub_futs = (0..sub_cpu_num)
        .into_par_iter()
        .map(|core_idx| {
            let sub_futures = (0..sub_per_peer_num)
                .into_par_iter()
                .map(|peer_index| {
                    subscribe_worker(
                        zenoh.clone(),
                        start_until,
                        timeout,
                        peer_index + core_idx * sub_per_peer_num,
                        tx.clone(),
                        args.multipeer_mode,
                        (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
                        args.locators.clone(),
                        args.clone(),
                        start,
                        None,
                        None,
                    )
                })
                .collect::<Vec<_>>();
            async_std::task::spawn(futures::future::join_all(sub_futures))
        })
        .collect::<Vec<_>>();
    let remaining_sub = total_sub_number % sub_cpu_num;
    let remaining_sub_fut = (total_sub_number - remaining_sub..total_sub_number)
        .into_par_iter()
        .map(|peer_index| {
            subscribe_worker(
                zenoh.clone(),
                start_until,
                timeout,
                peer_index,
                tx.clone(),
                args.multipeer_mode,
                (total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
                args.locators.clone(),
                args.clone(),
                start,
                None,
                None,
            )
        })
        .collect::<Vec<_>>();

    let remain_sub_futs = async_std::task::spawn(futures::future::join_all(remaining_sub_fut));
    sub_futs.push(remain_sub_futs);

    // Old publisher futures
    // let pub_futures = (0..total_put_number).map(|peer_index| {
    //     publish_worker(
    //         zenoh.clone(),
    //         start_until,
    //         timeout,
    //         peer_index,
    //         args.num_msgs_per_peer,
    //         &msg_payload,
    //     )
    // });
    // futures::future::try_join_all(pub_futures).await.unwrap();

    // new publisher futures
    let pub_cpu_num;
    if let Some(args_pub_cpu_num) = args.pub_cpu_num {
        pub_cpu_num = args_pub_cpu_num;
        if args_pub_cpu_num > (available_cpu_num + 1) / 2 {
            warn!("Spawning more than half of available cpu cores tasks for publishers.");
        }
    } else {
        pub_cpu_num = (available_cpu_num + 1) / 2;
    }
    println!("Will spawn up to {} tasks for publishers", pub_cpu_num + 1);

    let pub_per_peer_num = total_put_number / pub_cpu_num;
    let mut pub_futs = (0..pub_cpu_num)
        .into_par_iter()
        .map(|core_idx| {
            let pub_futures = (0..pub_per_peer_num)
                .into_par_iter()
                .map(|peer_index| {
                    publish_worker(
                        zenoh.clone(),
                        start_until,
                        timeout,
                        peer_index + core_idx * pub_per_peer_num,
                        args.num_msgs_per_peer,
                        get_msg_payload(args.payload_size, peer_index),
                        args.multipeer_mode,
                        args.locators.clone(),
                        args.output_dir.clone(),
                        total_put_number,
                        args.payload_size,
                        args.clone(),
                        start,
                        None,
                        None,
                    )
                })
                .collect::<Vec<_>>();
            async_std::task::spawn(futures::future::join_all(pub_futures))
        })
        .collect::<Vec<_>>();
    let remaining = total_put_number % pub_cpu_num;
    let remaining_fut = (total_put_number - remaining..total_put_number)
        .into_par_iter()
        .map(|peer_index| {
            publish_worker(
                zenoh.clone(),
                start_until,
                timeout,
                peer_index,
                args.num_msgs_per_peer,
                get_msg_payload(args.payload_size, peer_index),
                args.multipeer_mode,
                args.locators.clone(),
                args.output_dir.clone(),
                total_put_number,
                args.payload_size,
                args.clone(),
                start,
                None,
                None,
            )
        })
        .collect::<Vec<_>>();
    let remain_futs = async_std::task::spawn(futures::future::join_all(remaining_fut));

    pub_futs.push(remain_futs);

    let all_sub_fut = futures::future::join_all(sub_futs);

    let all_pub_fut = futures::future::join_all(pub_futs);

    let demo_fut = demonstration_worker(
        rx,
        total_put_number,
        total_sub_number,
        args.num_msgs_per_peer,
        args.remote_pub_peers,
        args.payload_size,
        args.round_timeout,
        args.clone(),
    );

    drop(tx);

    futures::join!(all_pub_fut, all_sub_fut, demo_fut);
    let zenoh = Arc::try_unwrap(zenoh).ok().unwrap();
    zenoh.close().await.unwrap();
}
