use crate::Opts;
use anyhow::{anyhow, ensure, Result};
use async_std::{stream::interval, task::sleep};
use futures::{future::FutureExt, stream, stream::TryStreamExt, StreamExt};
use output_config::{Cli, TestResult};
use rand::{prelude::*, rngs::OsRng};
use reliable_broadcast as rb;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use zenoh as zn;

const KEY: &str = "/key";

pub async fn run(config: &Opts) -> Result<TestResult> {
    ensure!(
        !config.cli.pub_sub_separate,
        "pub_sub_separate must be false"
    );

    let Opts {
        cli:
            Cli {
                payload_size,
                peer_id,
                num_msgs_per_peer,
                total_put_number,
                remote_pub_peers,
                ..
            },
        congestion_control,
        reliability,
        sub_mode,
        max_rounds,
        ..
    } = *config;
    let num_expected = (total_put_number + remote_pub_peers) * num_msgs_per_peer;
    let init_time = Duration::from_millis(config.cli.init_time);
    let round_timeout = Duration::from_millis(config.cli.round_timeout);
    let echo_interval = Duration::from_millis(config.echo_interval);
    let publish_interval = (round_timeout * max_rounds as u32) + Duration::from_millis(50);

    let session = zn::open(zn::config::Config::default())
        .await
        .map_err(|err| anyhow!("{}", err))?;
    let session = Arc::new(session);

    let (sender, stream) = rb::Config {
        max_rounds: config.max_rounds,
        extra_rounds: config.extra_rounds,
        round_timeout,
        echo_interval,
        congestion_control,
        reliability,
        sub_mode,
    }
    .build(session.clone(), KEY)
    .await
    .map_err(|err| anyhow!("{}", err))?;

    let instant = Instant::now();

    let producer_future = async move {
        sleep(init_time).await;

        stream::once(async move { () })
            .chain(interval(publish_interval))
            .take(num_msgs_per_peer)
            .enumerate()
            .map(anyhow::Ok)
            .try_fold(sender, move |sender, (seq, ())| async move {
                let data: u8 = OsRng.gen();
                eprintln!("{} sends seq={}, data={}", peer_id, seq, data);
                sender.send(data).await.map_err(|err| anyhow!("{}", err))?;
                anyhow::Ok(sender)
            })
            .await?;

        anyhow::Ok(())
    };

    let consumer_future = async move {
        let consumer_timeout =
            publish_interval * num_msgs_per_peer as u32 + Duration::from_millis(500);

        let num_received = stream
            .take(num_expected)
            .take_until({
                async move {
                    sleep(consumer_timeout).await;
                    eprintln!("{} timeout", peer_id);
                }
            })
            .try_fold(0, |cnt, event| async move {
                let rb::Event {
                    result,
                    broadcast_id,
                } = event;

                match result {
                    Ok(data) => {
                        eprintln!(
                            "{} accepted data={} for broadcast_id={}",
                            peer_id, data, broadcast_id
                        );

                        Ok(cnt + 1)
                    }
                    Err(err) => {
                        eprintln!(
                            "{} failed broadcast_id={} due to error: {:?}",
                            peer_id, broadcast_id, err
                        );
                        Ok(cnt + 1)
                    }
                }
            })
            .await
            .map_err(|err| anyhow!("{}", err))?;

        if num_received < num_expected {
            let lost_msgs = num_expected - num_received;
            let lost_pct = lost_msgs as f64 / num_expected as f64;
            eprintln!(
                "{} lost {} broadcast messages ({:.2}%).",
                peer_id,
                lost_msgs,
                lost_pct * 100.0
            );
        }

        anyhow::Ok((num_received, num_expected))
    };

    futures::try_join!(producer_future, consumer_future)?;

    let elapsed = instant.elapsed();
    let session = Arc::try_unwrap(session).expect("please report bug");
    session.close().await.map_err(|err| anyhow!("{}", err))?;

    Ok(TestResult {
        config: config.cli.clone(),
        total_sub_returned: todo!(),
        total_receive_rate: todo!(),
        per_peer_result: todo!(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpLog {
    // pub config: Experiment,
    pub receive_rate: f64,
    pub average_time: f64,
}
