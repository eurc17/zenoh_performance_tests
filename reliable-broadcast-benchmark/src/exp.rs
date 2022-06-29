use std::collections::HashMap;

use crate::{common::*, msg::Msg, utils::sleep_until};
use async_std::{stream::interval, task::sleep};
use once_cell::sync::OnceCell;
use output_config::{get_msg_payload, Cli, PeerResult, PubPeerResult};
use rb::config::IoConfig;
use reliable_broadcast as rb;
use rustdds::DomainParticipant;
use serde_loader::Json5Path;
use zn::config::{ConnectConfig, EndPoint};

pub async fn run(config: &Cli) -> Result<PeerResult, Error> {
    assert!(!config.pub_sub_separate, "pub_sub_separate must be false");

    let (sender, stream) = {
        let io_config_file = config.rb_io.as_ref().unwrap();
        let io_config: IoConfig = Json5Path::open_and_take(&io_config_file)?;

        // Create Zenoh session
        let zenoh_session = if io_config.is_zenoh() {
            let mut zn_config = zn::config::default();
            let endpoints: Vec<_> = config
                .locators
                .iter()
                .cloned()
                .map(EndPoint::from)
                .collect();
            zn_config.set_connect(ConnectConfig { endpoints }).unwrap();
            zn_config.set_add_timestamp(Some(true)).unwrap();
            let session = zn::open(zn_config).await?;
            Some(Arc::new(session))
        } else {
            None
        };

        // Create DDS domain participant
        let dds_domain_participant = if io_config.is_rust_dds() {
            static DDS_DOMAIN_PARTICIPANT: OnceCell<DomainParticipant> = OnceCell::new();

            let part = DDS_DOMAIN_PARTICIPANT.get_or_try_init(|| DomainParticipant::new(0))?;
            Some(part)
        } else {
            None
        };

        rb::Config {
            max_rounds: config.max_rounds.unwrap(),
            extra_rounds: config.extra_rounds.unwrap(),
            round_timeout: config.round_interval(),
            echo_interval: config.echo_interval(),
            io: io_config,
        }
        .build(zenoh_session, dds_domain_participant)
        .await?
    };

    let start_time = config.start_time_from_now();
    let producer_future = producer(config, start_time, sender);
    let consumer_future = consumer(config, start_time, stream);
    let ((), report) = futures::try_join!(producer_future, consumer_future)?;

    Ok(report)
}

async fn producer(config: &Cli, start_time: Instant, sender: rb::Sender<Msg>) -> Result<()> {
    let Cli {
        payload_size,
        peer_id,
        num_msgs_per_peer,
        ..
    } = *config;
    let publish_interval = config.publish_interval();
    let test_timeout = config.test_timeout();
    let msg = {
        let payload = get_msg_payload(payload_size, peer_id as u64);
        Msg(payload)
    };

    sleep_until(start_time).await;

    stream::once(future::ready(()))
        .chain(interval(publish_interval))
        .take_until(sleep(test_timeout))
        .take(num_msgs_per_peer)
        .enumerate()
        .map(ZOk)
        .try_fold(sender, move |sender, (seq, ())| {
            let msg = msg.clone();

            async move {
                debug!("peer {} sends seq={}", peer_id, seq);
                sender.send(msg).await?;
                ZOk(sender)
            }
        })
        .await?;

    debug!("peer {} producer finished", peer_id);

    Ok(())
}

async fn consumer(
    config: &Cli,
    start_time: Instant,
    stream: impl Stream<Item = Result<rb::Event<Msg>, Error>> + Send,
) -> Result<PeerResult> {
    struct Stat {
        pub num_accepted: usize,
        pub num_rejected: usize,
        pub broadcasters: HashMap<u64, BroadcasterStat>, // key is peer_id
    }

    #[derive(Clone)]
    struct BroadcasterStat {
        pub num_msgs: usize,
        pub total_rounds: usize,
        pub latencies: Vec<Duration>,
    }

    let num_expected = config.total_msg_num();
    let test_timeout = config.test_timeout();

    let mut stream = stream
        .take_until({
            async move {
                sleep_until(start_time + test_timeout).await;
                // debug!("peer {} timeout", config.peer_id);
            }
        })
        .boxed();

    // loop
    let mut stat = {
        Stat {
            num_accepted: 0,
            num_rejected: 0,
            broadcasters: HashMap::new(),
        }
    };

    // sleep_until(start_time).await;

    while let Some(event) = stream.try_next().await? {
        let rb::Event {
            result,
            latency,
            num_rounds,
            ..
        } = event;

        let msg = match result {
            Ok(msg) => {
                stat.num_accepted += 1;
                msg
            }
            Err(_) => {
                stat.num_rejected += 1;
                continue;
            }
        };

        let peer_id = output_config::peer_id_from_payload(msg.as_ref());

        use std::collections::hash_map::Entry as E;
        let peer_stat = match stat.broadcasters.entry(peer_id) {
            E::Occupied(entry) => entry.into_mut(),
            E::Vacant(entry) => entry.insert(BroadcasterStat {
                num_msgs: 0,
                latencies: vec![],
                total_rounds: 0,
            }),
        };

        peer_stat.num_msgs += 1;
        peer_stat.latencies.push(latency);
        peer_stat.total_rounds += num_rounds;

        if stat.num_accepted == num_expected {
            break;
        }
    }

    let elapsed_seconds = start_time.elapsed().as_secs_f64();
    let average_rb_rounds = stat
        .broadcasters
        .values()
        .map(|peer_stat| peer_stat.total_rounds)
        .sum::<usize>() as f64
        / stat.num_accepted as f64;
    let result_vec = stat
        .broadcasters
        .iter()
        .map(|(&peer_id, peer_stat)| {
            let throughput = peer_stat.num_msgs as f64 / elapsed_seconds;
            let average_latency = peer_stat.latencies.iter().cloned().sum::<Duration>()
                / peer_stat.latencies.len() as u32;
            let key_expr = format!("/demo/example/{}", peer_id);

            PubPeerResult {
                key_expr,
                throughput,
                average_latency_ms: average_latency.as_secs_f64() * 1000.0,
                average_rb_rounds: Some(peer_stat.total_rounds as f64 / peer_stat.num_msgs as f64),
            }
        })
        .collect();

    debug!("peer {} consumer finished", config.peer_id);

    Ok(PeerResult {
        short_config: Some(config.into()),
        peer_id: config.peer_id,
        receive_rate: stat.num_accepted as f64 / num_expected as f64,
        recvd_msg_num: stat.num_accepted,
        expected_msg_num: num_expected,
        average_rb_rounds: Some(average_rb_rounds),
        result_vec,
    })
}
