use std::{sync::Arc, time::SystemTime};

use super::sample::KeyedSample;
use anyhow::{Error, Result};
use blocking::unblock;
use cyclonedds_rs::DdsWriter;
use futures::{sink, Sink};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Sender<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub(crate) key: Uuid,
    pub(crate) writer: Option<DdsWriter<KeyedSample<T>>>,
}

impl<T> Sender<T>
where
    T: 'static + Send + Serialize + for<'de> Deserialize<'de>,
{
    pub async fn send(&mut self, data: T) -> Result<()> {
        let mut writer = self.writer.take().unwrap();
        let key = self.key;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let writer = unblock(move || -> Result<_> {
            let sample = KeyedSample {
                key,
                timestamp,
                data,
            };
            writer.write(Arc::new(sample))?;
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
