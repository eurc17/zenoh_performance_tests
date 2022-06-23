use crate::{common::*, message::*, state::State};

/// Start a worker that consumes input messages and handle each message accordingly.
pub(crate) async fn run_receiving_worker<T>(
    state: Arc<State<T>>,
    receiver: crate::io::generic::Receiver<Message<T>>,
) -> Result<(), Error>
where
    T: MessageT,
{
    // The lifetime of subscriber must be longer than the stream.
    // Otherwise, the stream is unable to retrieve input message.
    receiver
        .into_sample_stream()
        .try_filter_map(|sample| async move {
            let value = match sample.to_value()? {
                Some(value) => value,
                None => return Ok(None),
            };
            Ok(Some((sample, value)))
        })
        .try_for_each_concurrent(8, move |(sample, msg)| {
            let state = state.clone();

            async move {
                match msg {
                    Message::Broadcast(msg) => state.handle_broadcast(sample, msg),
                    Message::Present(msg) => state.handle_present(sample, msg),
                    Message::Echo(msg) => state.handle_echo(sample, msg),
                }
                Ok(())
            }
        })
        .await?;

    Ok(())
}

/// Start a worker that periodically publishes batched echos.
pub(crate) async fn run_echo_worker<T>(state: Arc<State<T>>) -> Result<(), Error>
where
    T: MessageT,
{
    async_std::stream::interval(state.echo_interval)
        .map(Ok)
        .try_for_each(move |()| {
            let state = state.clone();

            async move {
                let echo_requests = {
                    let mut echo_requests = state.echo_requests.write().await;
                    mem::take(&mut *echo_requests)
                };
                let broadcast_ids: Vec<_> = echo_requests.into_iter().collect();
                let msg: Message<T> = Echo {
                    from: state.my_id,
                    broadcast_ids,
                }
                .into();
                let mut io_sender = state.io_sender.lock().await;
                io_sender.send(msg).await?;
                Result::<(), Error>::Ok(())
            }
        })
        .await?;

    Ok(())
}
