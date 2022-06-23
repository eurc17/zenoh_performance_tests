use async_std::sync::Mutex;

use crate::{
    common::*,
    config::IoConfig,
    io::generic::Sample,
    message::*,
    worker::{run_echo_worker, run_receiving_worker},
    Config, ConsensusError, Event,
};
use async_std::task::spawn;
use uhlc::HLC;

pub(crate) struct State<T>
where
    T: MessageT,
{
    pub(crate) my_id: Uuid,
    pub(crate) seq_number: AtomicUsize,
    pub(crate) active_peers: DashSet<Uuid>,
    pub(crate) echo_requests: RwLock<DashSet<BroadcastId>>,
    pub(crate) contexts: DashMap<BroadcastId, BroadcastContext>,
    pub(crate) pending_echos: DashMap<BroadcastId, Arc<DashSet<Uuid>>>,
    pub(crate) commit_tx: flume::Sender<Event<T>>,
    /// The maximum number of rounds to run the reliable broadcast.
    pub(crate) max_rounds: usize,
    /// The number of extra rounds to send echo(m,s). It will not exceed the `max_rounds`
    pub(crate) extra_rounds: usize,
    /// The timeout for each round. Must be larger than 2 * `recv_timeout`.
    pub(crate) round_timeout: Duration,
    pub(crate) echo_interval: Duration,
    pub(crate) hlc: HLC,
    pub(crate) io_sender: Mutex<crate::io::Sender<Message<T>>>,
}

impl<T> State<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone,
{
    pub(crate) async fn new(
        config: &Config,
        zenoh_session: Option<Arc<zenoh::Session>>,
        dds_domain_participant: Option<&rustdds::DomainParticipant>,
    ) -> Result<
        (
            crate::sender::Sender<T>,
            impl Stream<Item = Result<Event<T>, Error>> + Send,
        ),
        Error,
    >
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone,
    {
        // sanity check
        if config.echo_interval >= config.round_timeout {
            return Err(anyhow!("echo_interval must be less than round_timeout").into());
        }
        if config.extra_rounds >= config.max_rounds {
            return Err(anyhow!("extra_rounds must be less than max_rounds").into());
        }

        let my_id = Uuid::new_v4();
        let (commit_tx, commit_rx) = flume::unbounded();

        let io_config = config.io.clone();

        let io_sender: crate::io::Sender<_>;
        let io_receiver: crate::io::Receiver<_>;
        (io_sender, io_receiver) = match &io_config {
            IoConfig::Zenoh(config) => {
                let session =
                    zenoh_session.ok_or_else(|| anyhow!("Zenoh session is not provided"))?;
                let zenoh_id: Uuid = session.id().await.parse()?;

                let sender = {
                    let send_key = format!("{}/{}", config.key, zenoh_id);

                    crate::io::zenoh::SenderConfig {
                        congestion_control: config.congestion_control.into(),
                        kind: Default::default(),
                    }
                    .build(session.clone(), send_key)
                    .await?
                };

                let receiver = {
                    let recv_key = format!("{}/*", config.key);

                    crate::io::zenoh::ReceiverConfig {
                        reliability: config.reliability.into(),
                        sub_mode: config.sub_mode.into(),
                    }
                    .build(session.clone(), &recv_key)
                    .await?
                };

                (sender.into(), receiver.into())
            }
            IoConfig::Dds(config) => {
                let domain_participant = dds_domain_participant
                    .ok_or_else(|| anyhow!("domain_participant is not provided"))?;

                let sender = config.build_sender(domain_participant)?;
                let receiver = config.build_receiver(domain_participant)?;
                (sender.into(), receiver.into())
            }
        };

        let state = Arc::new(State::<T> {
            my_id,
            seq_number: AtomicUsize::new(0),
            active_peers: DashSet::new(),
            echo_requests: RwLock::new(DashSet::new()),
            contexts: DashMap::new(),
            pending_echos: DashMap::new(),
            max_rounds: config.max_rounds,
            extra_rounds: config.extra_rounds,
            round_timeout: config.round_timeout,
            echo_interval: config.echo_interval,
            commit_tx,
            hlc: HLC::default(),
            io_sender: Mutex::new(io_sender),
        });

        let receiving_worker = spawn(run_receiving_worker(state.clone(), io_receiver));
        let echo_worker = spawn(run_echo_worker(state.clone()));
        let join_task = future::try_join(receiving_worker, echo_worker);

        let sender = crate::sender::Sender {
            state: state.clone(),
        };
        let event_stream = commit_rx.into_stream().then(move |event| {
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

        let select_stream = stream::select(
            join_task.map_ok(|_| None).into_stream(),
            event_stream.map(|event| Ok(Some(event))),
        )
        .try_filter_map(|data| async move { Ok(data) });

        Ok((sender, select_stream))
    }

    /// Schedule a future task to publish an echo.
    async fn request_sending_echo(self: Arc<Self>, broadcast_id: BroadcastId) {
        self.echo_requests.read().await.insert(broadcast_id);
    }

    /// Publish a broadcast.
    pub async fn broadcast(self: Arc<Self>, data: T) -> Result<(), Error> {
        let seq = self.seq_number.fetch_add(1, SeqCst);

        let msg: Message<T> = Broadcast {
            from: self.my_id,
            seq,
            data,
        }
        .into();
        let mut io_sender = self.io_sender.lock().await;
        io_sender.send(msg).await?;
        Ok(())
    }

    /// Process an input broadcast.
    pub(crate) fn handle_broadcast(self: Arc<Self>, sample: Sample<Message<T>>, msg: Broadcast<T>) {
        let broadcast_id = msg.broadcast_id();
        self.active_peers.insert(broadcast_id.broadcaster);

        debug!(
            "{} -> {}: broadcast, seq={}",
            broadcast_id.broadcaster, self.my_id, broadcast_id.seq
        );

        use dashmap::mapref::entry::Entry::*;
        match self.contexts.entry(broadcast_id) {
            Occupied(_) => {
                debug!(
                    "ignore duplicated broadcast for broadcast_id {}",
                    broadcast_id
                );
            }
            Vacant(entry) => {
                // remove related pending echos
                let acked_peers =
                    if let Some((_, acked_peers)) = self.pending_echos.remove(&broadcast_id) {
                        acked_peers
                    } else {
                        Arc::new(DashSet::new())
                    };

                let task = spawn(self.clone().run_broadcast_worker(
                    broadcast_id,
                    acked_peers.clone(),
                    sample,
                    msg.data,
                ));

                let context = BroadcastContext {
                    acked: acked_peers,
                    task,
                };

                entry.insert(context);
            }
        }
    }

    /// Process an input present message.
    pub(crate) fn handle_present(&self, _sample: Sample<Message<T>>, msg: Present) {
        let Present { from: sender } = msg;

        debug!("{} -> {}: present", sender, self.my_id);
        self.active_peers.insert(sender);
    }

    /// Process an input echo.
    pub(crate) fn handle_echo(&self, _sample: Sample<Message<T>>, msg: Echo) {
        let sender = msg.from;
        self.active_peers.insert(sender);

        msg.broadcast_ids.into_iter().for_each(|broadcast_id| {
            debug!(
                "{} -> {}: echo, broadcast={}",
                sender, self.my_id, broadcast_id,
            );

            match self.contexts.get(&broadcast_id) {
                Some(context) => {
                    // save the echoing peer id to corr. broadcast
                    context.acked.insert(sender);
                }
                None => {
                    info!(
                        "{} received echo from {} for broadcast_id {}, \
                         but broadcast was not received",
                        self.my_id, sender, broadcast_id
                    );

                    // save the echo message
                    use dashmap::mapref::entry::Entry::*;

                    let acked_peers = match self.pending_echos.entry(broadcast_id) {
                        Occupied(entry) => entry.into_ref(),
                        Vacant(entry) => entry.insert(Arc::new(DashSet::new())),
                    };
                    acked_peers.insert(sender);
                }
            }
        });
    }

    /// Start a worker for a received broadcast.
    async fn run_broadcast_worker(
        self: Arc<Self>,
        broadcast_id: BroadcastId,
        acked_peers: Arc<DashSet<Uuid>>,
        sample: Sample<Message<T>>,
        data: T,
    ) {
        spawn(async move {
            let get_latency = || {
                let sample_timestamp = sample.timestamp();

                let ok = self.hlc.update_with_timestamp(&sample_timestamp).is_ok();
                assert!(ok, "timestamp drifts too much");
                let tagged_timestamp = self.hlc.new_timestamp();
                tagged_timestamp.get_diff_duration(&sample.timestamp())
            };

            // TODO: determine start time from timestamp in broadcast message
            let mut interval = async_std::stream::interval(self.round_timeout);

            // send echo
            self.clone().request_sending_echo(broadcast_id).await;

            let tuple = (&mut interval)
                .take(self.max_rounds)
                .enumerate()
                .filter_map(|(round, ())| {
                    let me = self.clone();
                    let acked_peers = acked_peers.clone();

                    async move {
                        debug!(
                            "{} finishes round {} for broadcast_id {}",
                            me.my_id, round, broadcast_id
                        );

                        let num_peers = me.active_peers.len();
                        let num_echos = acked_peers.len();

                        if num_peers >= 4 {
                            // case: n_echos >= 2/3 n_peers
                            if num_echos * 3 >= num_peers * 2 {
                                Some((round, Ok(())))
                            }
                            // case: n_echos >= 1/3 n_peers
                            else if num_echos * 3 >= num_peers {
                                // send echo and try again
                                me.request_sending_echo(broadcast_id).await;
                                None
                            }
                            // case: n_echos < 1/3 n_peers
                            else {
                                Some((round, Err(ConsensusError::InsufficientEchos)))
                            }
                        }
                        // case: n_peers < 4
                        else {
                            Some((round, Err(ConsensusError::InsufficientPeers)))
                        }
                    }
                })
                .boxed()
                .next()
                .await;

            match tuple {
                // accepted before max_roudns
                Some((last_round, Ok(()))) => {
                    debug!(
                        "{} accepts a msg in round {} for broadcast_id {}",
                        self.my_id, last_round, broadcast_id
                    );

                    // trigger event
                    let event = Event {
                        result: Ok(data),
                        broadcast_id,
                        latency: get_latency(),
                        num_rounds: last_round + 1,
                    };
                    let _ = self.commit_tx.send_async(event).await;

                    // unconditionally send echo for more extra rounds
                    let extra_rounds =
                        cmp::min(self.extra_rounds, self.max_rounds - last_round - 1);

                    interval
                        .take(extra_rounds)
                        .enumerate()
                        .for_each(|(round, ())| {
                            let me = self.clone();

                            async move {
                                debug!(
                                    "{} runs extra round {} for broadcast_id {}",
                                    me.my_id,
                                    round + last_round + 1,
                                    broadcast_id
                                );
                                me.request_sending_echo(broadcast_id).await;
                            }
                        })
                        .await;
                }
                // error before max_roudns
                Some((last_round, Err(err))) => {
                    debug!(
                        "{} rejects the msg for broadcast_id {} due to {:?}",
                        self.my_id, broadcast_id, err
                    );

                    let event = Event {
                        result: Err(err),
                        broadcast_id,
                        latency: get_latency(),
                        num_rounds: last_round + 1,
                    };
                    let _ = self.commit_tx.send_async(event).await;
                }
                // not accepted when reaching max_rounds
                None => {
                    debug!(
                        "{} rejects the msg for broadcast_id {} due to reach max_round",
                        self.my_id, broadcast_id
                    );

                    let event = Event {
                        result: Err(ConsensusError::ConsensusLost),
                        broadcast_id,
                        latency: get_latency(),
                        num_rounds: self.max_rounds,
                    };
                    let _ = self.commit_tx.send_async(event).await;
                }
            }
        })
        .await
    }
}

/// The context for a broadcast.
pub struct BroadcastContext {
    /// The set of peers that replies echos.
    pub acked: Arc<DashSet<Uuid>>,
    /// The task handle to the broadcast worker.
    pub task: JoinHandle<()>,
}
