use std::convert::TryInto;
use zenoh::*;

use anyhow::Result;
use futures::{self, prelude::*};
use log::*;
use rayon::prelude::*;
use std::{sync::*, time::*};

#[async_std::main]
async fn main() {
    let zenoh = Arc::new(Zenoh::new(net::config::default()).await.unwrap());
    let start = Instant::now();
    let start_until = start + Duration::from_millis(100);
    let timeout = start_until + Duration::from_millis(100);
    let total_sub_number = 10;

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
    let total_put_number = 100000;
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

async fn publish_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
) -> Result<()> {
    async_std::task::sleep(start_until - Instant::now()).await;
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

async fn subscribe_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
    peer_id: usize,
) -> (usize, Vec<Change>) {
    async_std::task::sleep(start_until - Instant::now()).await;
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
