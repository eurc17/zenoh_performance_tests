use anyhow::{Error, Result};
use blocking::unblock;
use futures::{sink, Sink};
use rustdds::{no_key::DataWriter, CDRSerializerAdapter};
use serde::Serialize;

pub struct Sender<T>
where
    T: Serialize,
{
    pub(crate) writer: Option<DataWriter<T, CDRSerializerAdapter<T>>>,
}

impl<T> Sender<T>
where
    T: 'static + Send + Serialize,
{
    pub async fn send(&mut self, data: T) -> Result<()> {
        let writer = self.writer.take().unwrap();
        let writer = unblock(move || -> Result<_> {
            writer.write(data, None)?;
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
