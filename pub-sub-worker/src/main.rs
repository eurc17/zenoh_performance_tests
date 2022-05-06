mod common;
mod workers;
use common::*;
use output_config::get_msg_payload;
use output_config::Cli;
use workers::*;

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    // Get & parse arguments
    let args = Cli::parse();
    let default_wait_time = (10 * args.total_put_number as u64).max(2000);

    async_std::task::sleep(Duration::from_millis(
        default_wait_time - args.delay_startup,
    ))
    .await;

    // Parameters
    let start = Instant::now();
    println!("Peer {}, start = {:?}", args.peer_id, start);
    let process_start = datetime::Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);

    // Start workers
    let pub_sub_fut = pub_and_sub_worker(
        start_until,
        timeout,
        args.peer_id,
        args.num_msgs_per_peer,
        get_msg_payload(args.payload_size, args.peer_id),
        (args.total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
        args.locators.clone(),
        args.output_dir.clone(),
        args.total_put_number,
        args.payload_size,
        args.clone(),
        start,
        process_start,
    );
    let _result = futures::join!(pub_sub_fut);
}
