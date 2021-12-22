use super::common::*;

pub async fn publish_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
) -> Result<()> {
    let curr_time = Instant::now();
    if start_until > curr_time {
        async_std::task::sleep(start_until - curr_time).await;
    }
    let workspace = zenoh.workspace(None).await.unwrap();
    let msg_payload = format!("Hello World from peer {:08}", peer_id);
    workspace
        .put(
            &"/demo/example/hello".try_into().unwrap(),
            msg_payload.into(),
        )
        .await
        .unwrap();
    if timeout <= Instant::now() {
        warn!("publish worker sent message after timeout! Please reduce # of publishers or increase timeout.");
    }
    Ok(())
}

pub async fn subscribe_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
) -> (usize, Vec<Change>) {
    let mut change_vec = vec![];
    let workspace = zenoh.workspace(None).await.unwrap();
    let mut change_stream = workspace
        .subscribe(&"/demo/example/**".try_into().unwrap())
        .await
        .unwrap();
    while let Some(change) = change_stream.next().await {
        // println!(
        //     ">> {:?} for {} : {:?} at {}",
        //     change.kind, change.path, change.value, change.timestamp
        // );
        if Instant::now() < timeout {
            change_vec.push(change);
        } else {
            println!("Timeout reached");
            break;
        }
    }
    (peer_id, change_vec)
}
