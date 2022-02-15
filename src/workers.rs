use crate::{
    utils::{PeerResult, TestResult},
    Cli,
};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::{io::Write, str::FromStr};

use super::common::*;

pub async fn demonstration_worker(
    rx: flume::Receiver<(usize, Vec<Sample>)>,
    total_put_number: usize,
    total_sub_number: usize,
    num_msgs_per_peer: usize,
    additional_pub_num: usize,
    payload_size: usize,
    round_timeout: u64,
    args: Cli,
) -> () {
    let mut vector_data = vec![];
    while let Ok(data) = rx.recv_async().await {
        vector_data.push(data);
    }
    println!(
        "Received data from {}/{} sub peers",
        vector_data.len(),
        total_sub_number
    );
    vector_data.par_sort_by_key(|k| k.0);
    let total_msg_num = (total_put_number + additional_pub_num) * num_msgs_per_peer;

    let peer_result = vector_data
        .par_iter()
        .map(|(id, change_vec)| {
            println!(
                "sub peer {}: total received messages: {}/{}",
                id,
                change_vec.len(),
                total_msg_num
            );
            PeerResult {
                peer_id: *id,
                receive_rate: (change_vec.len() as f64) / (total_msg_num as f64),
                recvd_msg_num: change_vec.len(),
                expected_msg_num: total_msg_num,
            }
        })
        .collect::<Vec<_>>();
    let total_received_msgs = vector_data
        .par_iter()
        .map(|(_, change_vec)| change_vec.len())
        .sum::<usize>();
    let total_receive_rate =
        (total_received_msgs as f64) / (vector_data.len() as f64 * total_msg_num as f64);
    let file_path = args.output_dir.join(format!(
        "{}-{}-{}-{}-{}-{}.json",
        total_put_number,
        total_sub_number,
        num_msgs_per_peer,
        payload_size,
        round_timeout,
        args.init_time
    ));
    let test_result = TestResult {
        config: args,
        total_sub_returned: vector_data.len(),
        total_receive_rate,
        per_peer_result: peer_result,
    };

    let mut file = std::fs::File::create(file_path).unwrap();
    writeln!(
        &mut file,
        "{}",
        serde_json::to_string_pretty(&test_result).unwrap()
    )
    .unwrap();
}

pub async fn publish_worker(
    zenoh: Arc<Session>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    num_msgs_per_peer: usize,
    msg_payload: String,
    multipeer_mode: bool,
    locators: Option<String>,
    output_dir: PathBuf,
    total_put_number: usize,
    payload_size: usize,
    args: Cli,
    start: Instant,
    session_start_time: Option<Instant>,
    pub_sub_worker_start: Option<Instant>,
) -> Result<()> {
    let start_worker = Instant::now() - start;
    let zenoh_new;
    let mut timeout_flag = false;
    let before_sending;
    let start_sending;
    let after_sending;
    let mut session_start = session_start_time;
    if multipeer_mode {
        let mut config = config::default();
        if let Some(locators) = locators {
            let locator_vec = locators
                .split(",")
                .filter(|str| *str != "")
                .map(|locator| Locator::from_str(locator).unwrap())
                .collect::<Vec<_>>();
            config.set_peers(locator_vec).unwrap();
        }
        zenoh_new = zenoh::open(config).await.unwrap();
        session_start = Some(Instant::now());
        let curr_time = Instant::now();
        before_sending = curr_time - start;
        if start_until > curr_time {
            async_std::task::sleep(start_until - curr_time).await;
        }
        start_sending = Instant::now() - start;
        info!("start sending messages");
        for _ in 0..num_msgs_per_peer {
            zenoh_new
                .put("/demo/example/hello", msg_payload.clone())
                .await
                .unwrap();
            if timeout <= Instant::now() {
                timeout_flag = true;
                warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
                break;
            }
        }
        after_sending = Instant::now() - start;
        zenoh_new.close().await.unwrap();
    } else {
        let curr_time = Instant::now();
        before_sending = curr_time - start;
        if start_until > curr_time {
            async_std::task::sleep(start_until - curr_time).await;
        }
        start_sending = Instant::now() - start;
        info!("start sending messages");
        for _ in 0..num_msgs_per_peer {
            zenoh
                .put("/demo/example/hello", msg_payload.clone())
                .await
                .unwrap();
            if timeout <= Instant::now() {
                timeout_flag = true;
                warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
                break;
            }
        }
        after_sending = Instant::now() - start;
    }

    if timeout_flag {
        let file_path = output_dir.join(format!("info-{}.txt", peer_id));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
            .unwrap();
        writeln!(
            &mut file,
            "Peer-{} publisher timeout. Put_peer_num = {}, payload_size = {}",
            peer_id, total_put_number, payload_size,
        )
        .unwrap();
    }

    let file_path = args.output_dir.join(format!(
        "put_{}_info-{}-{}-{}-{}-{}-{}.json",
        peer_id,
        total_put_number,
        args.num_put_peer,
        num_msgs_per_peer,
        payload_size,
        args.round_timeout,
        args.init_time
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap();
    writeln!(&mut file, "start_worker: {} ms", start_worker.as_millis()).unwrap();
    if let Some(session_start) = session_start {
        writeln!(
            &mut file,
            "session_start: {} ms",
            (session_start - start).as_millis()
        )
        .unwrap();
    }
    if let Some(pub_sub_worker_start) = pub_sub_worker_start {
        writeln!(
            &mut file,
            "pub_sub_worker_start: {} ms",
            (pub_sub_worker_start - start).as_millis()
        )
        .unwrap();
    }
    writeln!(
        &mut file,
        "before_sending: {} ms",
        before_sending.as_millis()
    )
    .unwrap();
    writeln!(&mut file, "start_sending: {} ms", start_sending.as_millis()).unwrap();
    writeln!(&mut file, "after_sending: {} ms", after_sending.as_millis()).unwrap();

    Ok(())
}

pub async fn subscribe_worker(
    zenoh: Arc<Session>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    tx: flume::Sender<(usize, Vec<Sample>)>,
    multipeer_mode: bool,
    total_msg_num: usize,
    locators: Option<String>,
    args: Cli,
    start: Instant,
    session_start_time: Option<Instant>,
    pub_sub_worker_start: Option<Instant>,
) -> Result<()> {
    let start_worker = Instant::now() - start;
    let mut change_vec = vec![];
    let after_subscribing;
    let start_receiving;
    let after_receiving;
    let mut session_start = session_start_time;

    if start_until < Instant::now() {
        warn!("Subscriber is not initialized after the initial time has passed. Please increase initialization time");
        tx.send_async((peer_id, change_vec.clone())).await.unwrap();
        return Ok(());
    }
    let zenoh_new;
    if multipeer_mode {
        let mut config = config::default();
        if let Some(locators) = locators {
            let locator_vec = locators
                .split(",")
                .filter(|str| *str != "")
                .map(|locator| Locator::from_str(locator).unwrap())
                .collect::<Vec<_>>();
            config.set_peers(locator_vec).unwrap();
        }
        zenoh_new = zenoh::open(config).await.unwrap();
        session_start = Some(Instant::now());
        {
            let mut subscriber = zenoh_new.subscribe("/demo/example/**").await.unwrap();
            after_subscribing = Instant::now() - start;
            let stream = subscriber.receiver();
            start_receiving = Instant::now() - start;
            change_vec = stream
                .take(total_msg_num)
                .take_until({
                    async move {
                        let now = Instant::now();
                        if timeout >= now {
                            async_std::task::sleep(timeout - Instant::now()).await;
                        } else {
                            async_std::task::sleep(timeout - timeout).await;
                        }
                    }
                })
                .collect::<Vec<Sample>>()
                .await;
            after_receiving = Instant::now() - start;
            tx.send_async((peer_id, change_vec)).await.unwrap();
        }
        zenoh_new.close().await.unwrap();
    } else {
        let mut subscriber = zenoh.subscribe("/demo/example/**").await.unwrap();
        after_subscribing = Instant::now() - start;
        let stream = subscriber.receiver();
        start_receiving = Instant::now() - start;
        change_vec = stream
            .take(total_msg_num)
            .take_until({
                async move {
                    let now = Instant::now();
                    if timeout >= now {
                        async_std::task::sleep(timeout - Instant::now()).await;
                    } else {
                        async_std::task::sleep(timeout - timeout).await;
                    }
                }
            })
            .collect::<Vec<Sample>>()
            .await;
        after_receiving = Instant::now() - start;
        tx.send_async((peer_id, change_vec)).await.unwrap();
    }
    let file_path = args.output_dir.join(format!(
        "sub_{}_info-{}-{}-{}-{}-{}-{}.json",
        peer_id,
        args.num_put_peer,
        args.num_put_peer,
        args.num_msgs_per_peer,
        args.payload_size,
        args.round_timeout,
        args.init_time
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap();
    writeln!(&mut file, "start_worker: {} ms", start_worker.as_millis()).unwrap();
    if let Some(session_start) = session_start {
        writeln!(
            &mut file,
            "session_start: {} ms",
            (session_start - start).as_millis()
        )
        .unwrap();
    }
    if let Some(pub_sub_worker_start) = pub_sub_worker_start {
        writeln!(
            &mut file,
            "pub_sub_worker_start: {} ms",
            (pub_sub_worker_start - start).as_millis()
        )
        .unwrap();
    }
    writeln!(
        &mut file,
        "before_sending: {} ms",
        after_subscribing.as_millis()
    )
    .unwrap();
    writeln!(
        &mut file,
        "start_sending: {} ms",
        start_receiving.as_millis()
    )
    .unwrap();
    writeln!(
        &mut file,
        "after_sending: {} ms",
        after_receiving.as_millis()
    )
    .unwrap();
    Ok(())
}

pub async fn pub_and_sub_worker(
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    num_msgs_per_peer: usize,
    msg_payload: String,
    tx: flume::Sender<(usize, Vec<Sample>)>,
    total_msg_num: usize,
    locators: Option<String>,
    output_dir: PathBuf,
    total_put_number: usize,
    payload_size: usize,
    args: Cli,
    start: Instant,
) -> Result<()> {
    let pub_sub_worker_start = Some(Instant::now());
    let mut config = config::default();
    if let Some(locators) = locators.clone() {
        let locator_vec = locators
            .split(",")
            .filter(|str| *str != "")
            .map(|locator| Locator::from_str(locator).unwrap())
            .collect::<Vec<_>>();
        config.set_peers(locator_vec).unwrap();
    }
    let zenoh = Arc::new(zenoh::open(config).await.unwrap());
    let session_start_time = Some(Instant::now());
    let pub_future = publish_worker(
        zenoh.clone(),
        start_until,
        timeout,
        peer_id,
        num_msgs_per_peer,
        msg_payload,
        false,
        locators.clone(),
        output_dir,
        total_put_number,
        payload_size,
        args.clone(),
        start,
        session_start_time,
        pub_sub_worker_start,
    );
    let sub_future = subscribe_worker(
        zenoh.clone(),
        start_until,
        timeout,
        peer_id,
        tx,
        false,
        total_msg_num,
        locators.clone(),
        args.clone(),
        start,
        session_start_time,
        pub_sub_worker_start,
    );
    futures::try_join!(pub_future, sub_future)?;
    let zenoh = Arc::try_unwrap(zenoh).map_err(|_| ()).unwrap();
    zenoh.close().await.unwrap();

    Ok(())
}
