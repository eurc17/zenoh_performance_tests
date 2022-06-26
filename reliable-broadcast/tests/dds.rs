use std::{collections::HashMap, time::Duration};

use anyhow::{ensure, Result};
use async_std::task::{sleep, spawn};
use futures::{future, StreamExt, TryStreamExt};
use itertools::Itertools as _;
use reliable_broadcast::io::dds::Config;
use rustdds::{DomainParticipant, GUID};
use serde_loader::Json5Path;

const PUBLISHER_DELAY: Duration = Duration::from_millis(1000);

#[async_std::test]
pub async fn dds_one_to_one_test() -> Result<()> {
    type Msg = u64;
    const NUM_VALUES: u64 = 100;

    let config: Config = Json5Path::open_and_take(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/dds_one_to_one.json5"
    ))?;
    let domain_participant = DomainParticipant::new(0)?;
    let mut tx = config.build_sender::<Msg>(&domain_participant)?;
    let mut rx = config.build_receiver::<Msg>(&domain_participant)?;

    let pub_task = spawn(async move {
        sleep(PUBLISHER_DELAY).await;

        for val in 0..NUM_VALUES {
            tx.send(val).await?;
        }
        anyhow::Ok(())
    });

    let sub_task = spawn(async move {
        let mut num_received = 0;

        for expect in 0..NUM_VALUES {
            let val = rx.recv().await?;
            let val = match val {
                Some(val) => val,
                None => break,
            };
            ensure!(val == expect, "expect {}, but received {}", expect, val);
            num_received += 1;
        }

        ensure!(
            num_received == NUM_VALUES,
            "expect {} values, but received {} values",
            NUM_VALUES,
            num_received
        );
        anyhow::Ok(())
    });

    future::try_join(pub_task, sub_task).await?;

    Ok(())
}

#[async_std::test]
pub async fn dds_one_to_many_test() -> Result<()> {
    type Msg = u64;
    const NUM_VALUES: u64 = 100;
    const NUM_READERS: usize = 5;

    let config: Config = Json5Path::open_and_take(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/dds_one_to_many.json5"
    ))?;
    let domain_participant = DomainParticipant::new(0)?;
    let mut tx = config.build_sender::<Msg>(&domain_participant)?;

    let pub_task = spawn(async move {
        sleep(PUBLISHER_DELAY).await;

        for val in 0..NUM_VALUES {
            tx.send(val).await?;
        }
        anyhow::Ok(())
    });

    let sub_tasks: Vec<_> = (0..NUM_READERS)
        .map(|reader_idx| -> Result<_> {
            let mut rx = config.build_receiver::<Msg>(&domain_participant)?;

            let future = spawn(async move {
                let mut num_received = 0;

                for expect in 0..NUM_VALUES {
                    let val = rx.recv().await?;
                    let val = match val {
                        Some(val) => val,
                        None => break,
                    };
                    ensure!(
                        val == expect,
                        "reader {}: expect {}, but received {}",
                        reader_idx,
                        expect,
                        val
                    );
                    num_received += 1;
                }

                ensure!(
                    num_received == NUM_VALUES,
                    "reader {}: expect {} values, but received {} values",
                    reader_idx,
                    NUM_VALUES,
                    num_received
                );
                anyhow::Ok(())
            });

            Ok(future)
        })
        .try_collect()?;

    future::try_join(pub_task, future::try_join_all(sub_tasks)).await?;

    Ok(())
}

#[async_std::test]
pub async fn dds_many_to_many_test() -> Result<()> {
    type Msg = u64;
    const NUM_VALUES: u64 = 100;
    const NUM_WRITERS: usize = 5;
    const NUM_READERS: usize = 5;

    let config: Config = Json5Path::open_and_take(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/dds_one_to_many.json5"
    ))?;
    let domain_participant = DomainParticipant::new(0)?;

    let pub_tasks: Vec<_> = (0..NUM_WRITERS)
        .map(|_writer_idx| -> Result<_> {
            let mut tx = config.build_sender::<Msg>(&domain_participant)?;

            let future = spawn(async move {
                sleep(PUBLISHER_DELAY).await;

                for val in 0..NUM_VALUES {
                    tx.send(val).await?;
                }
                anyhow::Ok(())
            });

            Ok(future)
        })
        .try_collect()?;

    let sub_tasks: Vec<_> = (0..NUM_READERS)
        .map(|reader_idx| -> Result<_> {
            let rx = config.build_receiver::<Msg>(&domain_participant)?;

            let future = spawn(async move {
                let msgs: Vec<_> = rx
                    .into_sample_stream()
                    .take(NUM_WRITERS * NUM_VALUES as usize)
                    .map(|s| anyhow::Ok(s?.into_value().unwrap()))
                    .try_collect()
                    .await?;

                let groups: HashMap<GUID, Vec<u64>> = msgs
                    .into_iter()
                    .map(|msg| (msg.key, msg.data))
                    .into_group_map();

                ensure!(
                    groups.len() == NUM_WRITERS,
                    "reader {} only finds {} writers",
                    reader_idx,
                    groups.len()
                );
                ensure!(
                    groups
                        .values()
                        .all(|numbers| numbers.len() == NUM_VALUES as usize),
                    "reader {} does not receive complete data",
                    reader_idx
                );

                anyhow::Ok(())
            });

            Ok(future)
        })
        .try_collect()?;

    future::try_join(
        future::try_join_all(pub_tasks),
        future::try_join_all(sub_tasks),
    )
    .await?;

    Ok(())
}
