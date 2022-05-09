use collected::SumVal;
use futures::stream::{self, StreamExt as _, TryStreamExt as _};
use rand::{prelude::*, rngs::OsRng};
use reliable_broadcast as rb;
use serde::{Deserialize, Serialize};
use std::{
    error::Error as StdError,
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use zenoh as zn;

type Error = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestConfig {
    pub num_peers: usize,
    pub num_msgs: usize,
    pub zenoh_key: String,
    #[serde(with = "humantime_serde")]
    pub round_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub echo_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub publisher_startup_delay: Duration,
    pub max_rounds: usize,
    pub extra_rounds: usize,
    pub sub_mode: rb::SubMode,
    pub reliability: rb::Reliability,
    pub congestion_control: rb::CongestionControl,
}

#[async_std::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let TestConfig {
        num_peers,
        num_msgs,
        zenoh_key,
        round_timeout,
        echo_interval,
        max_rounds,
        extra_rounds,
        congestion_control,
        reliability,
        sub_mode,
        publisher_startup_delay,
    } = {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("config.json5");
        let text = fs::read_to_string(path)?;
        json5::from_str(&text)?
    };

    // let startup_duration = Duration::from_millis(3000);
    let num_expected = num_msgs * num_peers;
    let interval = (round_timeout * max_rounds as u32) + Duration::from_millis(50);

    let futures = (0..num_peers).map(|_| {
        let zenoh_key = zenoh_key.clone();

        async_std::task::spawn(async move {
            let mut config = zn::config::default();
            config.set_add_timestamp(true.into()).unwrap();
            let session = Arc::new(zenoh::open(config).await?);
            let my_id = session.id().await;

            let (sender, stream) = rb::Config {
                max_rounds,
                extra_rounds,
                round_timeout,
                echo_interval,
                congestion_control,
                reliability,
                sub_mode,
            }
            .build(session, zenoh_key)
            .await?;
            let sink = sender.into_sink();

            let producer_task = {
                let my_id = my_id.clone();

                async move {
                    async_std::task::sleep(publisher_startup_delay).await;

                    stream::once(async move { () })
                        .chain(async_std::stream::interval(interval))
                        .take(num_msgs)
                        .enumerate()
                        .map(move |(seq, ())| {
                            let data: u8 = OsRng.gen();
                            eprintln!("{} sends seq={}, data={}", my_id, seq, data);
                            Ok(data)
                        })
                        .forward(sink)
                        .await?;

                    Result::<_, Error>::Ok(())
                }
            };

            let consumer_task = {
                let my_id = my_id.clone();

                async move {
                    let timeout = interval * num_msgs as u32 + Duration::from_millis(500);

                    let num_received = stream
                        .take(num_peers * num_msgs)
                        .take_until({
                            let my_id = my_id.clone();

                            async move {
                                async_std::task::sleep(timeout).await;
                                eprintln!("{} timeout", my_id);
                            }
                        })
                        .try_fold(0, |cnt, event| {
                            let my_id = my_id.clone();

                            async move {
                                let rb::Event {
                                    result,
                                    broadcast_id,
                                } = event;

                                match result {
                                    Ok(data) => {
                                        eprintln!(
                                            "{} accepted data={} for broadcast_id={}",
                                            my_id, data, broadcast_id
                                        );

                                        Ok(cnt + 1)
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "{} failed broadcast_id={} due to error: {:?}",
                                            my_id, broadcast_id, err
                                        );
                                        Ok(cnt + 1)
                                    }
                                }
                            }
                        })
                        .await?;

                    if num_received < num_msgs {
                        let lost_msgs = num_expected - num_received;
                        let lost_pct = lost_msgs as f64 / num_expected as f64;
                        eprintln!(
                            "{} lost {} broadcast messages ({:.2}%).",
                            my_id,
                            lost_msgs,
                            lost_pct * 100.0
                        );
                    }

                    Result::<_, Error>::Ok((num_received, num_expected))
                }
            };

            let instant = Instant::now();
            let ((), (num_received, num_expected)) =
                futures::future::try_join(producer_task, consumer_task).await?;
            eprintln!("elapsed time for {}: {:?}", my_id, instant.elapsed());

            Result::<_, Error>::Ok((num_received, num_expected))
        })
    });

    let start = Instant::now();
    let (total_received, total_expected): (SumVal<_>, SumVal<_>) =
        futures::future::try_join_all(futures)
            .await?
            .into_iter()
            .unzip();

    eprintln!("total elapsed time {:?}", start.elapsed());
    let total_expected = total_expected.into_inner();
    let total_received = total_received.into_inner();
    let total_lost = total_expected - total_received;
    let loss_rate = total_lost as f64 / total_expected as f64;
    eprintln!(
        "expect {} msgs, {} lost, loss rate {:.2}%",
        total_expected,
        total_lost,
        loss_rate * 100.0
    );

    Ok(())
}
