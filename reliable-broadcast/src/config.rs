use async_std::sync::Mutex;

use uhlc::HLC;
use zn::prelude::KeyExpr;

use crate::{common::*, sender::Sender, state::State, stream::Event};
use zenoh as zn;

#[derive(Debug, Clone, PartialEq)]
/// Defines the structure of the config file.
pub struct Config {
    /// The maximum number of rounds to run the reliable broadcast.
    pub max_rounds: usize,
    /// The number of extra rounds to send echo(m,s). It will not exceed the `max_rounds`
    pub extra_rounds: usize,
    /// The timeout for each round.
    pub round_timeout: Duration,
    /// The interval that publishes echo messages.
    pub echo_interval: Duration,
    /// I/O configuration.
    pub io: IoConfig,
}

impl Config {
    pub async fn build<'a, T, K>(
        &self,
        session: Arc<zn::Session>,
        key: K,
    ) -> Result<
        (
            Sender<T>,
            impl Stream<Item = Result<Event<T>, Error>> + Send,
        ),
        Error,
    >
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync,
        K: Into<KeyExpr<'a>>,
    {
        // sanity check
        if self.echo_interval >= self.round_timeout {
            return Err(anyhow!("echo_interval must be less than round_timeout").into());
        }
        if self.extra_rounds >= self.max_rounds {
            return Err(anyhow!("extra_rounds must be less than max_rounds").into());
        }

        let my_id = session.id().await.parse()?;
        let key = key.into().to_owned();
        let (commit_tx, commit_rx) = flume::unbounded();

        let io_config = self.io.clone();

        let io_sender: crate::io::Sender<_> = {
            match &io_config {
                IoConfig::Zenoh(config) => {
                    let send_key = format!("{}/{}", key, my_id);
                    crate::io::zenoh::SenderConfig {
                        congestion_control: config.congestion_control.into(),
                        kind: Default::default(),
                    }
                    .build(session.clone(), send_key)
                    .await?
                    .into()
                }
                IoConfig::Dds(_) => todo!(),
            }
        };

        let state = Arc::new(State::<T> {
            key,
            my_id,
            seq_number: AtomicUsize::new(0),
            active_peers: DashSet::new(),
            echo_requests: RwLock::new(DashSet::new()),
            contexts: DashMap::new(),
            pending_echos: DashMap::new(),
            session,
            max_rounds: self.max_rounds,
            extra_rounds: self.extra_rounds,
            round_timeout: self.round_timeout,
            echo_interval: self.echo_interval,
            commit_tx,
            // sub_mode: self.sub_mode,
            // reliability: self.reliability,
            io_config,
            hlc: HLC::default(),
            io_sender: Mutex::new(io_sender),
        });
        let receiving_worker = state.clone().run_receiving_worker();
        let echo_worker = state.clone().run_echo_worker();

        let sender = Sender {
            state: state.clone(),
        };
        let stream = {
            let stream = commit_rx.into_stream().then(move |event| {
                let state = state.clone();

                async move {
                    state
                        .contexts
                        .remove(&event.broadcast_id)
                        .unwrap()
                        .1
                        .task
                        .await;
                    event
                }
            });

            stream::select(
                future::try_join(receiving_worker, echo_worker)
                    .map_ok(|_| None)
                    .into_stream(),
                stream.map(|event| Ok(Some(event))),
            )
            .try_filter_map(|data| async move { Ok(data) })
        };

        Ok((sender, stream))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IoConfig {
    Zenoh(zenoh_config::ZenohConfig),
    Dds(crate::io::dds::Config),
}

pub mod zenoh_config {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub struct ZenohConfig {
        pub sub_mode: SubMode,
        pub reliability: Reliability,
        pub congestion_control: CongestionControl,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SubMode {
        Push,
        Pull,
    }

    impl From<SubMode> for zn::subscriber::SubMode {
        fn from(from: SubMode) -> Self {
            use zn::subscriber::SubMode as O;
            use SubMode as I;

            match from {
                I::Push => O::Push,
                I::Pull => O::Pull,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Reliability {
        BestEffort,
        Reliable,
    }

    impl From<Reliability> for zn::subscriber::Reliability {
        fn from(from: Reliability) -> Self {
            use zn::subscriber::Reliability as O;
            use Reliability as I;

            match from {
                I::BestEffort => O::BestEffort,
                I::Reliable => O::Reliable,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CongestionControl {
        Block,
        Drop,
    }

    impl From<CongestionControl> for zn::publication::CongestionControl {
        fn from(from: CongestionControl) -> Self {
            use zn::publication::CongestionControl as O;
            use CongestionControl as I;

            match from {
                I::Block => O::Block,
                I::Drop => O::Drop,
            }
        }
    }
}
