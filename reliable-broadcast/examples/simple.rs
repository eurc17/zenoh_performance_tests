use async_std::{
    stream::interval,
    task::{sleep, spawn},
};
use collected::SumVal;
use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
    Sink, Stream,
};
use once_cell::sync::OnceCell;
use rand::{prelude::*, rngs::OsRng};
use rb::Event;
use reliable_broadcast::{self as rb, config::IoConfig};
use rustdds::DomainParticipant;
use serde::{Deserialize, Serialize};
use serde_loader::Json5Path;
use std::{
    error::Error as StdError,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;
use zenoh as zn;

type Error = Box<dyn StdError + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

static DDS_DOMAIN_PARTICIPANT: OnceCell<DomainParticipant> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub num_peers: usize,
    pub num_msgs: usize,
    #[serde(with = "humantime_serde")]
    pub round_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub echo_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub publisher_startup_delay: Duration,
    pub max_rounds: usize,
    pub extra_rounds: usize,
    pub io: Json5Path<IoConfig>,
}

impl TestConfig {
    pub fn interval_timeout(&self) -> Duration {
        (self.round_timeout * self.max_rounds as u32) + Duration::from_millis(30)
    }

    pub fn num_total_msgs(&self) -> usize {
        self.num_msgs * self.num_peers
    }

    pub fn consumer_timeout(&self) -> Duration {
        self.interval_timeout() * self.num_msgs as u32
            + self.publisher_startup_delay
            + Duration::from_millis(1000)
    }
}

#[async_std::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    // Load config file
    let config: TestConfig = {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/config.json5");
        Json5Path::open_and_take(path)?
    };
    let config = Arc::new(config);
    let num_peers = config.num_peers;

    // Run RB peers
    let futures = (0..num_peers).map(|_| spawn(run(config.clone())));
    let start = Instant::now();
    let (total_received, total_expected): (SumVal<_>, SumVal<_>) =
        future::try_join_all(futures).await?.into_iter().unzip();
    let elapsed = start.elapsed();

    // Compute loss rate
    eprintln!("total elapsed time {:?}", elapsed);
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

async fn run(test_config: Arc<TestConfig>) -> Result<(usize, usize), Error> {
    let TestConfig {
        round_timeout,
        echo_interval,
        max_rounds,
        extra_rounds,
        ..
    } = *test_config;

    // Build RB sender/receiver
    let (sender, stream) = {
        let io_config = &*test_config.io;

        // Create Zenoh session
        let zenoh_session = if io_config.is_zenoh() {
            let mut config = zn::config::default();
            config.set_add_timestamp(true.into()).unwrap();
            let session = zenoh::open(config).await?;
            Some(Arc::new(session))
        } else {
            None
        };

        // Create DDS domain participant
        let dds_domain_participant = if io_config.is_rust_dds() {
            let part =
                DDS_DOMAIN_PARTICIPANT.get_or_try_init(|| rustdds::DomainParticipant::new(0))?;
            Some(part)
        } else {
            None
        };

        // Build RB sender/receiver
        rb::Config {
            max_rounds,
            extra_rounds,
            round_timeout,
            echo_interval,
            io: (*io_config).clone(),
        }
        .build(zenoh_session, dds_domain_participant)
        .await?
    };

    // Run producer/consumer tasks
    let my_id = sender.id();
    let producer_task = spawn(producer(test_config.clone(), my_id, sender.into_sink()));
    let consumer_task = spawn(consumer(test_config.clone(), my_id, stream));

    let instant = Instant::now();
    let ((), (num_received, num_expected)) = future::try_join(producer_task, consumer_task).await?;
    eprintln!("elapsed time for {}: {:?}", my_id, instant.elapsed());

    Ok((num_received, num_expected))
}

async fn producer<S>(config: Arc<TestConfig>, my_id: Uuid, sink: S) -> Result<()>
where
    S: Sink<u8, Error = Error>,
{
    let TestConfig {
        num_msgs,
        publisher_startup_delay,
        ..
    } = *config;
    let interval_timeout = config.interval_timeout();
    sleep(publisher_startup_delay).await;

    stream::once(async move {})
        .chain(interval(interval_timeout))
        .take(num_msgs)
        .enumerate()
        .map(move |(seq, ())| {
            let data: u8 = OsRng.gen();
            eprintln!("{} sends seq={}, data={}", my_id, seq, data);
            Ok(data)
        })
        .forward(sink)
        .await?;

    Ok(())
}

async fn consumer<S>(config: Arc<TestConfig>, my_id: Uuid, stream: S) -> Result<(usize, usize)>
where
    S: Stream<Item = Result<Event<u8>, Error>> + Send,
{
    let TestConfig { num_msgs, .. } = *config;
    let timeout = config.consumer_timeout();
    let num_expected = config.num_total_msgs();

    let num_received = stream
        .take(num_expected)
        .take_until(async move {
            sleep(timeout).await;
            eprintln!("{} timeout", my_id);
        })
        .try_fold(0, |cnt, event| async move {
            let rb::Event {
                result,
                broadcast_id,
                latency,
                num_rounds,
            } = event;

            match result {
                Ok(data) => {
                    eprintln!(
                        "{} accepted data={} for broadcast_id={} latency={:?} n_rounds={}",
                        my_id, data, broadcast_id, latency, num_rounds
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

    Ok((num_received, num_expected))
}
