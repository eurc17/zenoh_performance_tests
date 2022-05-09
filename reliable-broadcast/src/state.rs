use crate::{common::*, message::*, ConsensusError, Event};
use async_std::task::spawn;
use uhlc::HLC;
use zenoh::{
    publication::CongestionControl,
    subscriber::{Reliability, SubMode},
};

const ENCODING: Encoding = Encoding::APP_JSON;

pub(crate) struct State<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync,
{
    pub(crate) session: Arc<zn::Session>,
    pub(crate) my_id: Uuid,
    pub(crate) seq_number: AtomicUsize,
    pub(crate) key: KeyExpr<'static>,
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
    pub(crate) sub_mode: SubMode,
    pub(crate) reliability: Reliability,
    pub(crate) congestion_control: CongestionControl,
    pub(crate) hlc: HLC,
}

impl<T> State<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync,
{
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
        let value: Value = serde_json::to_value(&msg)?.into();
        self.session
            .put(format!("{}/{}", self.key, self.my_id), value)
            .congestion_control(self.congestion_control)
            .kind(SampleKind::Put)
            .encoding(Encoding::TEXT_PLAIN)
            .await?;
        Ok(())
    }

    /// Process an input broadcast.
    fn handle_broadcast(
        self: Arc<Self>,
        sample: Sample,
        tagged_timestamp: uhlc::Timestamp,
        msg: Broadcast<T>,
    ) {
        // TODO: check timestamp
        // let peer_id = sample.source_info.source_id.unwrap();
        // let seq = sample.source_info.source_sn.unwrap();

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
                    tagged_timestamp,
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
    fn handle_present(&self, _sample: Sample, msg: Present) {
        let Present { from: sender } = msg;
        // TODO: check timestamp
        // let peer_id = sample.source_info.source_id.unwrap();

        debug!("{} -> {}: present", sender, self.my_id);
        self.active_peers.insert(sender);
    }

    /// Process an input echo.
    fn handle_echo(&self, _sample: Sample, msg: Echo) {
        // TODO: check timestamp
        // let peer_id = sample.source_info.source_id.unwrap();

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

    /// Start a worker that consumes input messages and handle each message accordingly.
    pub async fn run_receiving_worker(self: Arc<Self>) -> Result<(), Error> {
        let me = self.clone();
        let subscriber_builder = me.session.subscribe(format!("{}/**", self.key));
        let mut subscriber = subscriber_builder
            .reliability(self.reliability)
            .mode(self.sub_mode)
            .await?;
        let receiver = subscriber.receiver().clone();
        let future = receiver
            .filter_map(|mut sample| async move {
                if sample.kind != SampleKind::Put {
                    return None;
                }

                sample.value = sample.value.encoding(ENCODING);

                guard!(let Some(value) = sample.value.as_json() else {
                    debug!("unable to decode message: not JSON format");
                    return None;
                });

                let value: Message<T> = match serde_json::from_value(value) {
                    Ok(value) => value,
                    Err(err) => {
                        debug!("unable to decode message: {:?}", err);
                        return None;
                    }
                };

                Some((sample, value))
            })
            .map(Result::<_, Error>::Ok)
            .try_for_each_concurrent(8, move |(sample, msg)| {
                let me = self.clone();

                async move {
                    let sample_timestamp = sample.timestamp.ok_or_else(|| {
                        Error::from(anyhow!("HLC feature must be enabled for Zenoh"))
                    })?;
                    let tagged_timestamp = {
                        let ok = me.hlc.update_with_timestamp(&sample_timestamp).is_ok();
                        if !ok {
                            return Err(anyhow!("timestamp drifts too much").into());
                        }
                        me.hlc.new_timestamp()
                    };

                    match msg {
                        Message::Broadcast(msg) => {
                            me.handle_broadcast(sample, tagged_timestamp, msg)
                        }
                        Message::Present(msg) => me.handle_present(sample, msg),
                        Message::Echo(msg) => me.handle_echo(sample, msg),
                    }
                    Ok(())
                }
            });

        spawn(future).await?;

        Ok(())
    }

    /// Start a worker that periodically publishes batched echos.
    pub async fn run_echo_worker(self: Arc<Self>) -> Result<(), Error> {
        spawn(async move {
            async_std::stream::interval(self.echo_interval)
                .map(Ok)
                .try_for_each(|()| {
                    let me = self.clone();

                    async move {
                        let echo_requests = {
                            let mut echo_requests = me.echo_requests.write().await;
                            mem::take(&mut *echo_requests)
                        };
                        let broadcast_ids: Vec<_> = echo_requests.into_iter().collect();
                        let msg: Message<T> = Echo {
                            from: me.my_id,
                            broadcast_ids,
                        }
                        .into();
                        let value: Value = serde_json::to_value(&msg)?.into();
                        me.session
                            .put(&me.key, value)
                            .congestion_control(CongestionControl::Drop)
                            .kind(SampleKind::Put)
                            .encoding(ENCODING)
                            .await?;
                        Result::<(), Error>::Ok(())
                    }
                })
                .await?;

            Result::<(), Error>::Ok(())
        })
        .await?;
        Ok(())
    }

    /// Start a worker for a received broadcast.
    async fn run_broadcast_worker(
        self: Arc<Self>,
        broadcast_id: BroadcastId,
        acked_peers: Arc<DashSet<Uuid>>,
        sample: Sample,
        tagged_timestamp: uhlc::Timestamp,
        data: T,
    ) {
        spawn(async move {
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
                        latency: tagged_timestamp
                            .get_diff_duration(sample.timestamp.as_ref().unwrap()),
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
                Some((_, Err(err))) => {
                    debug!(
                        "{} rejects the msg for broadcast_id {}",
                        self.my_id, broadcast_id
                    );

                    let event = Event {
                        result: Err(err),
                        broadcast_id,
                        latency: tagged_timestamp
                            .get_diff_duration(sample.timestamp.as_ref().unwrap()),
                    };
                    let _ = self.commit_tx.send_async(event).await;
                }
                // not accepted when reaching max_rounds
                None => {
                    debug!(
                        "{} rejects the msg for broadcast_id {}",
                        self.my_id, broadcast_id
                    );

                    let event = Event {
                        result: Err(ConsensusError::ConsensusLost),
                        broadcast_id,
                        latency: tagged_timestamp
                            .get_diff_duration(sample.timestamp.as_ref().unwrap()),
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
