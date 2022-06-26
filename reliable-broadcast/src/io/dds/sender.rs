use super::sample::KeyedSample;
use anyhow::{Error, Result};
use blocking::unblock;
use futures::{sink, Sink};
use rustdds::{with_key::DataWriter, CDRSerializerAdapter, Timestamp, GUID};
use serde::Serialize;

pub struct Sender<T>
where
    T: Serialize,
{
    pub(crate) key: GUID,
    pub(crate) writer: Option<DataWriter<KeyedSample<T>, CDRSerializerAdapter<KeyedSample<T>>>>,
}

impl<T> Sender<T>
where
    T: 'static + Send + Serialize,
{
    pub async fn send(&mut self, data: T) -> Result<()> {
        let writer = self.writer.take().unwrap();
        let key = self.key;
        let writer = unblock(move || -> Result<_> {
            let sample = KeyedSample { key, data };
            writer.write(sample, Some(Timestamp::now()))?;
            Ok(writer)
        })
        .await?;
        self.writer = Some(writer);

        Ok(())
    }

    pub fn into_sink(self) -> impl Sink<T, Error = Error> {
        sink::unfold(self, |mut sender, data: T| async move {
            sender.send(data).await.map(|_| sender)
        })
    }
}
