use anyhow::Result;
use blocking::unblock;
use futures::{stream, Stream};
use rustdds::{
    with_key::{DataReader, DataSample},
    CDRDeserializerAdapter,
};
use serde::Deserialize;

use super::sample::KeyedSample;

pub struct Receiver<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub(crate) reader: Option<DataReader<KeyedSample<T>, CDRDeserializerAdapter<KeyedSample<T>>>>,
}

impl<T> Receiver<T>
where
    T: 'static + for<'de> Deserialize<'de> + Send,
{
    pub async fn recv_sample(&mut self) -> Result<Option<DataSample<KeyedSample<T>>>> {
        let mut reader = self.reader.take().unwrap();
        let (reader, sample) = unblock(move || -> Result<_> {
            let sample = reader.take_next_sample()?;
            Ok((reader, sample))
        })
        .await?;
        self.reader = Some(reader);

        Ok(sample)
    }

    pub async fn recv(&mut self) -> Result<Option<T>> {
        let sample = self.recv_sample().await?;
        let data = sample
            .map(|s| s.into_value().ok())
            .flatten()
            .map(|s| s.data);
        Ok(data)
    }

    pub fn into_sample_stream(self) -> impl Stream<Item = Result<DataSample<KeyedSample<T>>>> {
        stream::try_unfold(self, |mut rx| async move {
            let item = rx.recv_sample().await?;
            anyhow::Ok(item.map(|item| (item, rx)))
        })
    }
}
