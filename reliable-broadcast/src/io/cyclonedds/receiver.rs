use std::sync::Arc;

use anyhow::Result;
use blocking::unblock;
use cyclonedds_rs::DdsReader;
use futures::{stream, Stream};
use serde::{Deserialize, Serialize};

use super::sample::KeyedSample;

pub struct Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub(crate) reader: Option<DdsReader<KeyedSample<T>>>,
}

impl<T> Receiver<T>
where
    T: 'static + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    pub async fn recv_sample(&mut self) -> Result<Arc<KeyedSample<T>>> {
        let reader = self.reader.take().unwrap();

        let (reader, sample) = unblock(move || -> Result<_> {
            let msg: Arc<KeyedSample<T>> = reader.take1_now()?;
            Ok((reader, msg))
        })
        .await?;

        self.reader = Some(reader);

        Ok(sample)
    }

    pub async fn recv(&mut self) -> Result<T>
    where
        T: Clone,
    {
        let sample = self.recv_sample().await?;
        let data = sample.data.clone();
        Ok(data)
    }

    pub fn into_sample_stream(self) -> impl Stream<Item = Result<Arc<KeyedSample<T>>>> {
        stream::try_unfold(self, |mut rx| async move {
            let item = rx.recv_sample().await?;
            anyhow::Ok(Some((item, rx)))
        })
    }
}
