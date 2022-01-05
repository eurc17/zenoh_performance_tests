use crate::{
    utils::{PeerResult, TestResult},
    Cli,
};
use std::io::Write;

use super::common::*;

pub async fn demonstration_worker(
    rx: flume::Receiver<(usize, Vec<Change>)>,
    total_put_number: usize,
    total_sub_number: usize,
    num_msgs_per_peer: usize,
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
    let total_msg_num = total_put_number * num_msgs_per_peer;

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
        "{}-{}-{}.json",
        total_put_number, total_sub_number, num_msgs_per_peer
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
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    _peer_id: usize,
    num_msgs_per_peer: usize,
    msg_payload: String,
    multipeer_mode: bool,
    locators: Option<String>,
) -> Result<()> {
    let zenoh_new;
    let workspace;
    if multipeer_mode {
        let mut config = Properties::default();
        if let Some(locators) = locators {
            config.insert("peer".to_string(), locators);
        }
        zenoh_new = Zenoh::new(config.into()).await.unwrap();
        workspace = zenoh_new.workspace(None).await.unwrap();
        let curr_time = Instant::now();
        if start_until > curr_time {
            async_std::task::sleep(start_until - curr_time).await;
        }
        info!("start sending messages");
        for _ in 0..num_msgs_per_peer {
            workspace
                .put(
                    &"/demo/example/hello".try_into().unwrap(),
                    msg_payload.clone().into(),
                )
                .await
                .unwrap();
            if timeout <= Instant::now() {
                warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
                break;
            }
        }
        zenoh_new.close().await.unwrap();
    } else {
        workspace = zenoh.workspace(None).await.unwrap();
        let curr_time = Instant::now();
        if start_until > curr_time {
            async_std::task::sleep(start_until - curr_time).await;
        }
        info!("start sending messages");
        for _ in 0..num_msgs_per_peer {
            workspace
                .put(
                    &"/demo/example/hello".try_into().unwrap(),
                    msg_payload.clone().into(),
                )
                .await
                .unwrap();
            if timeout <= Instant::now() {
                warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
                break;
            }
        }
    }

    Ok(())
}

pub async fn subscribe_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    tx: flume::Sender<(usize, Vec<Change>)>,
    multipeer_mode: bool,
    total_msg_num: usize,
    locators: Option<String>,
) -> Result<()> {
    let mut change_vec = vec![];

    if start_until < Instant::now() {
        warn!("Subscriber is not initialized after the initial time has passed. Please increase initialization time");
        tx.send_async((peer_id, change_vec.clone())).await.unwrap();
        return Ok(());
    }
    let zenoh_new;
    let workspace;
    if multipeer_mode {
        let mut config = Properties::default();
        if let Some(locators) = locators {
            config.insert("peer".to_string(), locators);
        }
        zenoh_new = Zenoh::new(config.into()).await.unwrap();
        workspace = zenoh_new.workspace(None).await.unwrap();

        let stream = workspace.subscribe(&"/demo/example/**".try_into()?).await?;

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
            .filter(|change| future::ready(change.kind == zenoh::ChangeKind::Put))
            .collect::<Vec<Change>>()
            .await;
        tx.send_async((peer_id, change_vec)).await.unwrap();
        zenoh_new.close().await.unwrap();
    } else {
        workspace = zenoh.workspace(None).await.unwrap();

        let stream = workspace.subscribe(&"/demo/example/**".try_into()?).await?;

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
            .filter(|change| future::ready(change.kind == zenoh::ChangeKind::Put))
            .collect::<Vec<Change>>()
            .await;
        tx.send_async((peer_id, change_vec)).await.unwrap();
    }
    // let mut change_stream = workspace
    //     .subscribe(&"/demo/example/**".try_into().unwrap())
    //     .await
    //     .unwrap();
    // while let Some(change) = change_stream.next().await {
    //     // println!(
    //     //     "{} received >> {:?} for {} : {:?} at {}",
    //     //     peer_id, change.kind, change.path, change.value, change.timestamp
    //     // );
    //     // let string = format!("{:?}", change.value);
    //     if Instant::now() < timeout {
    //         change_vec.push(change);
    //         // println!("{}, change_vec.len = {}", peer_id, change_vec.len());
    //     } else {
    //         println!("{}, Timeout reached", peer_id);
    //         break;
    //     }
    // }
    Ok(())
}

pub async fn pub_and_sub_worker(
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    num_msgs_per_peer: usize,
    msg_payload: String,
    tx: flume::Sender<(usize, Vec<Change>)>,
    total_msg_num: usize,
    locators: Option<String>,
) -> Result<()> {
    let mut config = Properties::default();
    if let Some(locators) = locators.clone() {
        config.insert("peer".to_string(), locators);
    }
    let zenoh = Arc::new(Zenoh::new(config.into()).await?);
    let pub_future = publish_worker(
        zenoh.clone(),
        start_until,
        timeout,
        peer_id,
        num_msgs_per_peer,
        msg_payload,
        false,
        locators.clone(),
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
    );
    futures::try_join!(pub_future, sub_future)?;
    let zenoh = Arc::try_unwrap(zenoh).map_err(|_| ()).unwrap();
    zenoh.close().await?;

    Ok(())
}
