use std::convert::TryInto;
use zenoh::*;

use anyhow::Result;
use futures::{self, prelude::*};
use std::{sync::*, time::*};

#[async_std::main]
async fn main() {
    let zenoh = Arc::new(Zenoh::new(net::config::default()).await.unwrap());
    let start_until = Instant::now() + Duration::from_millis(100);
    let until = start_until + Duration::from_millis(100);
    let _timeout = until.saturating_duration_since(Instant::now());
    async_std::task::spawn(subscribe_worker(zenoh.clone(), start_until));
    async_std::task::sleep(std::time::Duration::from_millis(50)).await;
    // async_std::task::spawn(publish_worker(zenoh.clone(), start_until));
    let futures =
        (0..10000).map(|peer_index| publish_worker(zenoh.clone(), start_until, peer_index));
    futures::future::try_join_all(futures).await.unwrap();
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
}

async fn publish_worker(zenoh: Arc<Zenoh>, start_until: Instant, peer_id: usize) -> Result<()> {
    async_std::task::sleep(start_until - Instant::now()).await;
    let workspace = zenoh.workspace(None).await.unwrap();
    let msg_payload = format!("Hello World from peer {}", peer_id);
    workspace
        .put(
            &"/demo/example/hello".try_into().unwrap(),
            msg_payload.into(),
        )
        .await
        .unwrap();
    Ok(())
}

async fn subscribe_worker(zenoh: Arc<Zenoh>, start_until: Instant) {
    async_std::task::sleep(start_until - Instant::now()).await;
    let workspace = zenoh.workspace(None).await.unwrap();
    let mut change_stream = workspace
        .subscribe(&"/demo/example/**".try_into().unwrap())
        .await
        .unwrap();
    while let Some(change) = change_stream.next().await {
        println!(
            ">> {:?} for {} : {:?} at {}",
            change.kind, change.path, change.value, change.timestamp
        )
    }
}
