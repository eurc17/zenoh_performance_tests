use uhlc::HLC;

use crate::{common::*, sender::Sender, state::State, stream::Event};

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
    pub sub_mode: zenoh::subscriber::SubMode,
    pub reliability: zenoh::subscriber::Reliability,
    pub congestion_control: zenoh::publication::CongestionControl,
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

        let key = key.into().to_owned();
        let (commit_tx, commit_rx) = flume::unbounded();

        let state = Arc::new(State::<T> {
            key,
            my_id: session.id().await.parse()?,
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
            congestion_control: self.congestion_control,
            sub_mode: self.sub_mode,
            reliability: self.reliability,
            hlc: HLC::default(),
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
