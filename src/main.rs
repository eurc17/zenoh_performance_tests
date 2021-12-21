use std::convert::TryInto;
use zenoh::*;

use anyhow::Result;
use futures::{self, prelude::*};
use std::{sync::*, time::*};

#[async_std::main]
async fn main() {
    let zenoh = Arc::new(Zenoh::new(net::config::default()).await.unwrap());
    let start = Instant::now();
    let start_until = start + Duration::from_millis(100);
    let timeout = start_until + Duration::from_millis(100);
    let sub_handle = async_std::task::spawn(subscribe_worker(zenoh.clone(), start_until, timeout));
    async_std::task::sleep(std::time::Duration::from_millis(50)).await;
    // async_std::task::spawn(publish_worker(zenoh.clone(), start_until));
    let total_number = 100000;
    let pub_futures =
        (0..total_number).map(|peer_index| publish_worker(zenoh.clone(), start_until, peer_index));
    futures::future::try_join_all(pub_futures).await.unwrap();

    // async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    let result = async_std::future::timeout(Duration::from_millis(1000), sub_handle).await;
    if result.is_err() {
        println!("All messages delivered!");
    } else {
        // for change in result.unwrap().iter() {
        //     println!(
        //         ">> {:?} for {} : {:?} at {}",
        //         change.kind, change.path, change.value, change.timestamp
        //     );
        // }
        let change_vec = result.unwrap();
        println!(
            "total received messages: {}/{}",
            change_vec.len(),
            total_number
        );
    }
    let msg_payload = format!("Hello World from peer {:08}", 1 as usize);
    let payload_size = std::mem::size_of_val(&msg_payload);
    println!("payload size = {:?} bytes.", payload_size);
}

async fn publish_worker(zenoh: Arc<Zenoh>, start_until: Instant, peer_id: usize) -> Result<()> {
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
    Ok(())
}

async fn subscribe_worker(
    zenoh: Arc<Zenoh>,
    start_until: Instant,
    timeout: Instant,
) -> Vec<Change> {
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
    change_vec
}
