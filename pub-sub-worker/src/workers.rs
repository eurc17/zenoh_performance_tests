use crate::{
    utils::{
        PeerResult, PubPeerResult, PubPeerStatistics, PubTimeStatus, ShortConfig, SubTimeStatus,
        TestResult,
    },
    Cli,
};
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::OpenOptions};
use uhlc::HLC;

use super::common::*;

pub async fn _demonstration_worker(
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
                short_config: None,
                peer_id: *id,
                receive_rate: (change_vec.len() as f64) / (total_msg_num as f64),
                recvd_msg_num: change_vec.len(),
                expected_msg_num: total_msg_num,
                result_vec: vec![],
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
        "Exp_{}-{}-{}-{}-{}-{}.json",
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
    locators: Vec<Locator>,
    output_dir: PathBuf,
    total_put_number: usize,
    payload_size: usize,
    args: Cli,
    start: Instant,
    session_start_time: Option<Instant>,
    pub_sub_worker_start: Option<Instant>,
    _hlc: Arc<HLC>,
) -> Result<()> {
    let start_worker = Instant::now() - start;
    let zenoh_new;
    let mut timeout_flag = false;
    let before_sending;
    let start_sending;
    let after_sending;
    let mut session_start = session_start_time;
    if multipeer_mode {
        //default = false
        let mut config = config::default();
        let endpoints = locators
            .into_iter()
            .map(|locator| EndPoint::from(locator))
            .collect::<Vec<_>>();
        let listerner_config = ListenConfig { endpoints };
        config.set_listen(listerner_config).unwrap();
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
                .put(format!("/demo/example/{}", peer_id), msg_payload.clone())
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
                .put(format!("/demo/example/{}", peer_id), msg_payload.clone())
                .await
                .unwrap();
            if timeout <= Instant::now() {
                timeout_flag = true;
                warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
                break;
            }
            async_std::task::sleep(Duration::from_millis(args.pub_interval)).await;
        }
        after_sending = Instant::now() - start;
    }

    if timeout_flag {
        let file_path = output_dir.join(format!("info-{}.txt", peer_id));
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(file_path)
            .unwrap();
        writeln!(
            &mut file,
            "Peer-{} publisher timeout. Exp: {}-{}-{}-{}-{}-{}",
            peer_id,
            total_put_number,
            args.total_put_number,
            num_msgs_per_peer,
            payload_size,
            args.round_timeout,
            args.init_time
        )
        .unwrap();
    }

    std::fs::create_dir_all(&args.output_dir)?;

    let file_path = args.output_dir.join(format!(
        "put_{}_info_{}-{}-{}-{}-{}-{}.json",
        peer_id,
        total_put_number,
        args.total_put_number,
        num_msgs_per_peer,
        payload_size,
        args.round_timeout,
        args.init_time
    ));
    let session_start_millis = match session_start {
        Some(time) => Some((time - start).as_millis()),
        _ => None,
    };
    let pub_sub_worker_start_millis = match pub_sub_worker_start {
        Some(time) => Some((time - start).as_millis()),
        _ => None,
    };
    let pub_time_status = PubTimeStatus {
        start_pub_worker: start_worker.as_millis(),
        session_start: session_start_millis,
        pub_sub_worker_start: pub_sub_worker_start_millis,
        before_sending: before_sending.as_millis(),
        start_sending: start_sending.as_millis(),
        after_sending: after_sending.as_millis(),
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap();
    writeln!(
        &mut file,
        "{}",
        serde_json::to_string_pretty(&pub_time_status).unwrap()
    )
    .unwrap();

    Ok(())
}

pub async fn subscribe_worker(
    zenoh: Arc<Session>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    multipeer_mode: bool,
    total_msg_num: usize,
    locators: Vec<Locator>,
    args: Cli,
    start: Instant,
    session_start_time: Option<Instant>,
    pub_sub_worker_start: Option<Instant>,
    process_start: datetime::Instant,
    hlc: Arc<HLC>,
) -> Result<()> {
    let start_worker = Instant::now() - start;
    let change_vec;
    let after_subscribing;
    let start_receiving;
    let after_receiving;
    let mut session_start = session_start_time;

    if start_until < Instant::now() {
        warn!("Subscriber is not initialized after the initial time has passed. Please increase initialization time");
        // tx.send_async((peer_id, change_vec.clone())).await.unwrap();
        return Ok(());
    }
    let zenoh_new;
    if multipeer_mode {
        //default = false
        let mut config = config::default();
        let endpoints = locators
            .into_iter()
            .map(|locator| EndPoint::from(locator))
            .collect::<Vec<_>>();
        let listerner_config = ListenConfig { endpoints };
        config.set_listen(listerner_config).unwrap();
        zenoh_new = zenoh::open(config).await.unwrap();
        session_start = Some(Instant::now());
        {
            let mut subscriber = zenoh_new.subscribe("/demo/example/**").await.unwrap();
            after_subscribing = Instant::now() - start;
            let stream = subscriber.receiver();
            start_receiving = Instant::now() - start;
            change_vec = stream
                .then(|change| async {
                    if let Some(timestamp) = &change.timestamp {
                        hlc.update_with_timestamp(timestamp).unwrap();
                    }
                    let hlc_timestamp = hlc.new_timestamp();
                    (change, hlc_timestamp)
                })
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
                .collect::<Vec<(Sample, _)>>()
                .await;
            after_receiving = Instant::now() - start;
            // tx.send_async((peer_id, change_vec)).await.unwrap();
        }
        zenoh_new.close().await.unwrap();
    } else {
        let mut subscriber = zenoh.subscribe("/demo/example/**").await.unwrap();
        after_subscribing = Instant::now() - start;
        let stream = subscriber.receiver();
        start_receiving = Instant::now() - start;
        change_vec = stream
            .then(|change| async {
                if let Some(timestamp) = &change.timestamp {
                    hlc.update_with_timestamp(timestamp).unwrap();
                }
                let hlc_timestamp = hlc.new_timestamp();
                (change, hlc_timestamp)
            })
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
            .collect::<Vec<(Sample, _)>>()
            .await;
        after_receiving = Instant::now() - start;
        // tx.send_async((peer_id, change_vec)).await.unwrap();
    }
    let mut result_map: HashMap<String, PubPeerStatistics> = HashMap::new();
    for change in change_vec.iter() {
        let tagged_timestamp = change.0.timestamp.clone().unwrap();
        let time_diff_sub_pub = change.1.get_diff_duration(&tagged_timestamp);
        // dbg!(time_diff_sub_pub);
        // dbg!(&change.0.key_expr);
        let key_expr_string = change.0.key_expr.to_string();
        if key_expr_string == format!("/demo/example/{}", peer_id) {
            continue;
        }
        if result_map.get_mut(&key_expr_string).is_some() {
            let pub_peer_stat = result_map.get_mut(&key_expr_string).unwrap();
            pub_peer_stat.msg_cnt += 1;
            pub_peer_stat.latency_vec.push(time_diff_sub_pub);
        } else {
            let mut pub_peer_stat = PubPeerStatistics::new();
            pub_peer_stat.msg_cnt = 1;
            pub_peer_stat.latency_vec.push(time_diff_sub_pub);
            result_map.insert(key_expr_string, pub_peer_stat);
        }
    }
    let result_vec = result_map
        .into_iter()
        .map(|(key_expr, pub_peer_stat)| {
            let throughput =
                (pub_peer_stat.msg_cnt as f64) / (after_receiving - start_receiving).as_secs_f64();
            let latency_vec_ms = pub_peer_stat
                .latency_vec
                .iter()
                .map(|dur| (dur.as_micros() as f64) / 1000.0)
                .collect::<Vec<_>>();
            let average_latency_ms =
                latency_vec_ms.iter().sum::<f64>() / (latency_vec_ms.len() as f64);
            PubPeerResult {
                key_expr,
                throughput,
                average_latency_ms,
            }
        })
        .collect::<Vec<_>>();
    let short_config = ShortConfig {
        peer_id,
        total_put_number: args.total_put_number,
        num_msgs_per_peer: args.num_msgs_per_peer,
        payload_size: args.payload_size,
        round_timeout: args.round_timeout,
        init_time: args.init_time,
    };
    let peer_result = PeerResult {
        short_config: Some(short_config),
        peer_id,
        receive_rate: (change_vec.len() as f64) / (total_msg_num as f64),
        recvd_msg_num: change_vec.len(),
        expected_msg_num: total_msg_num,
        result_vec,
    };
    let file_path = args.output_dir.join(format!(
        "exp_sub_{}_{}-{}-{}-{}-{}-{}.json",
        peer_id,
        args.total_put_number,
        args.total_put_number,
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
    writeln!(
        &mut file,
        "{}",
        serde_json::to_string_pretty(&peer_result).unwrap()
    )
    .unwrap();

    let session_start_millis = match session_start {
        Some(time) => Some((time - start).as_millis()),
        _ => None,
    };
    let pub_sub_worker_start_millis = match pub_sub_worker_start {
        Some(time) => Some((time - start).as_millis()),
        _ => None,
    };

    let sub_time_status = SubTimeStatus {
        process_start_sec: process_start.seconds(),
        process_start_millis: process_start.milliseconds(),
        start_sub_worker: start_worker.as_millis(),
        session_start: session_start_millis,
        pub_sub_worker_start: pub_sub_worker_start_millis,
        after_subscribing: after_subscribing.as_millis(),
        start_receiving: start_receiving.as_millis(),
        after_receiving: after_receiving.as_millis(),
    };
    let file_path = args.output_dir.join(format!(
        "sub_{}_info_{}-{}-{}-{}-{}-{}.json",
        peer_id,
        args.total_put_number,
        args.total_put_number,
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
    writeln!(
        &mut file,
        "{}",
        serde_json::to_string_pretty(&sub_time_status).unwrap()
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
    total_msg_num: usize,
    locators: Vec<Locator>,
    output_dir: PathBuf,
    total_put_number: usize,
    payload_size: usize,
    args: Cli,
    start: Instant,
    process_start: datetime::Instant,
) -> Result<()> {
    let pub_sub_worker_start = Some(Instant::now());
    let mut config = config::default();
    let endpoints = locators
        .clone()
        .into_iter()
        .map(|locator| EndPoint::from(locator))
        .collect::<Vec<_>>();
    let connect_config = ConnectConfig { endpoints };
    config.set_connect(connect_config).unwrap();
    config.set_add_timestamp(Some(true)).unwrap();
    let zenoh = Arc::new(zenoh::open(config).await.unwrap());
    let session_start_time = Some(Instant::now());
    // Todo: Use the internel hlc in zenoh (not present for default config)
    let hlc = Arc::new(HLC::default());
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
        hlc.clone(),
    );
    let sub_future = subscribe_worker(
        zenoh.clone(),
        start_until,
        timeout,
        peer_id,
        false,
        total_msg_num,
        locators.clone(),
        args.clone(),
        start,
        session_start_time,
        pub_sub_worker_start,
        process_start,
        hlc,
    );
    if args.pub_disable {
        if args.sub_disable {
            println!("Cannot disable both publisher and subscriber. Will only disable publisher.");
        }
        futures::try_join!(sub_future)?;
        drop(pub_future);
    } else if args.sub_disable {
        futures::try_join!(pub_future)?;
        drop(sub_future);
    } else {
        futures::try_join!(pub_future, sub_future)?;
    }
    let zenoh = Arc::try_unwrap(zenoh).map_err(|_| ()).unwrap();
    zenoh.close().await.unwrap();

    Ok(())
}
