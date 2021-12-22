#![allow(unused)]

mod common;
mod workers;
use common::*;
use workers::*;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(
        short = "p",
        long,
        default_value = "1000",
        help = "The total number of publisher peers"
    )]
    num_put_peer: usize,
    #[structopt(
        short = "s",
        long,
        default_value = "10",
        help = "The total number of subscriber peers"
    )]
    num_sub_peer: usize,
    #[structopt(short = "t", long, default_value = "100")]
    /// The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).
    /// The subscriber will start receiving the messages at the same time as the publishers.
    round_timeout: u64,
    #[structopt(short = "i", long, default_value = "100")]
    /// The initial time for starting up futures.
    init_time: u64,
}
#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let args = Cli::from_args();
    dbg!(&args);
    let zenoh = Arc::new(Zenoh::new(net::config::default()).await.unwrap());
    let start = Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);
    let total_sub_number = args.num_sub_peer;
    let total_put_number = args.num_put_peer;

    let sub_handle_vec = (0..total_sub_number)
        .into_par_iter()
        .map(|peer_id: usize| {
            let sub_handle = async_std::task::spawn(subscribe_worker(
                zenoh.clone(),
                start_until,
                timeout,
                peer_id,
            ));
            sub_handle
        })
        .collect::<Vec<_>>();
    async_std::task::sleep(std::time::Duration::from_millis(50)).await;
    // async_std::task::spawn(publish_worker(zenoh.clone(), start_until));
    let pub_futures = (0..total_put_number)
        .map(|peer_index| publish_worker(zenoh.clone(), start_until, timeout, peer_index));
    futures::future::try_join_all(pub_futures).await.unwrap();

    // async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    let sub_handle_fut = sub_handle_vec
        .into_iter()
        .map(|sub_handle| async_std::future::timeout(Duration::from_millis(1000), sub_handle))
        .collect::<Vec<_>>();

    let result = futures::future::try_join_all(sub_handle_fut).await;

    // let result = async_std::future::timeout(Duration::from_millis(1000), sub_handle).await;
    if result.is_err() {
        println!("All messages delivered!");
    } else {
        // for change in result.unwrap().iter() {
        //     println!(
        //         ">> {:?} for {} : {:?} at {}",
        //         change.kind, change.path, change.value, change.timestamp
        //     );
        // }
        let result_vec = result.unwrap();
        for (id, change_vec) in result_vec.iter() {
            println!(
                "sub peer {}: total received messages: {}/{}",
                id,
                change_vec.len(),
                total_put_number
            );
        }
    }
    let msg_payload = format!("Hello World from peer {:08}", 1 as usize);
    let payload_size = std::mem::size_of_val(&msg_payload);
    println!("payload size = {:?} bytes.", payload_size);
}
