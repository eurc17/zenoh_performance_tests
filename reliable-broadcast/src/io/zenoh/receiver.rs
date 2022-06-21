use crate::common::*;
use ownref::ArcOwnedC;
use std::{marker::PhantomData, sync::Mutex};
use zenoh::{
    prelude::{Encoding, KeyExpr, Receiver as _, Sample, SampleKind},
    subscriber::{Reliability, SubMode, Subscriber},
    Session,
};

#[derive(Debug, Clone, Default)]
pub struct ReceiverConfig {
    pub reliability: Reliability,
    pub sub_mode: SubMode,
}

impl ReceiverConfig {
    pub async fn build<'a, T, K>(self, session: Arc<Session>, key_expr: K) -> Result<Receiver<T>>
    where
        K: Into<KeyExpr<'a>>,
        T: for<'de> Deserialize<'de>,
    {
        let session = ArcOwnedC::from_arc(session);
        let subscriber = session
            .try_then(|session| async move {
                let subscriber_builder = session.subscribe(key_expr);
                let subscriber = subscriber_builder
                    .reliability(self.reliability)
                    .mode(self.sub_mode)
                    .await?;
                let subscriber = Mutex::new(subscriber);
                Result::<_, Error>::Ok(subscriber)
            })
            .await?;

        Ok(Receiver {
            subscriber,
            _phantom: PhantomData,
        })
    }
}

pub struct Receiver<T>
where
    T: for<'de> Deserialize<'de>,
{
    subscriber: ArcOwnedC<'static, Session, Mutex<Subscriber<'static>>>,
    _phantom: PhantomData<T>,
}

impl<T> Receiver<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub async fn recv_sample(&mut self) -> Result<Sample> {
        let mut subscriber = self.subscriber.lock().unwrap();
        let sample = subscriber.receiver().recv_async().await?;
        Ok(sample)
    }

    pub async fn recv(&mut self) -> Result<T> {
        loop {
            let sample = self.recv_sample().await?;
            if sample.kind != SampleKind::Put {
                continue;
            }

            let value = sample.value.encoding(Encoding::APP_JSON);

            guard!(let Some(value) = value.as_json() else {
                debug!("unable to decode message: not JSON format");
                continue;
            });

            let value: T = serde_json::from_value(value)?;
            break Ok(value);
        }
    }

    pub fn into_sample_stream(self) -> impl Stream<Item = Sample> {
        stream::poll_fn(move |cx| {
            let mut subscriber = self.subscriber.lock().unwrap();
            subscriber.receiver().poll_next_unpin(cx)
        })
    }
}
