use super::common::*;

pub async fn demonstration_worker(
    rx: flume::Receiver<(usize, Vec<Change>)>,
    total_put_number: usize,
    total_sub_number: usize,
    num_msgs_per_peer: usize,
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
    for (id, change_vec) in vector_data.iter() {
        println!(
            "sub peer {}: total received messages: {}/{}",
            id,
            change_vec.len(),
            total_put_number * num_msgs_per_peer
        );
    }
}

pub async fn publish_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
    num_msgs_per_peer: usize,
    msg_payload: String,
    multipeer_mode: bool,
) -> Result<()> {
    let zenoh_new;
    let workspace;
    if multipeer_mode {
        zenoh_new = Zenoh::new(net::config::default()).await.unwrap();
        workspace = zenoh_new.workspace(None).await.unwrap();
    } else {
        workspace = zenoh.workspace(None).await.unwrap();
    }

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
        zenoh_new = Zenoh::new(net::config::default()).await.unwrap();
        workspace = zenoh_new.workspace(None).await.unwrap();
    } else {
        workspace = zenoh.workspace(None).await.unwrap();
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
    let stream = workspace.subscribe(&"/demo/example/**".try_into()?).await?;

    change_vec = stream
        .take(total_msg_num)
        .take_until({
            async move {
                let now = Instant::now();
                if timeout >= now {
                    async_std::task::sleep(timeout - Instant::now()).await;
                }
            }
        })
        .filter(|change| future::ready(change.kind == zenoh::ChangeKind::Put))
        .collect::<Vec<Change>>()
        .await;
    tx.send_async((peer_id, change_vec)).await.unwrap();
    Ok(())
}
