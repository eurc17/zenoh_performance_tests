use super::super::{cyclonedds as cdds_io, rustdds as rustdds_io, zenoh as zenoh_io};
use crate::common::*;

pub enum Sender<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    Zenoh(zenoh_io::Sender<T>),
    RustDds(rustdds_io::Sender<T>),
    CycloneDds(cdds_io::Sender<T>),
}

impl<T> From<cdds_io::Sender<T>> for Sender<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn from(v: cdds_io::Sender<T>) -> Self {
        Self::CycloneDds(v)
    }
}

impl<T> From<rustdds_io::Sender<T>> for Sender<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn from(v: rustdds_io::Sender<T>) -> Self {
        Self::RustDds(v)
    }
}

impl<T> From<zenoh_io::Sender<T>> for Sender<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn from(v: zenoh_io::Sender<T>) -> Self {
        Self::Zenoh(v)
    }
}

impl<T> Sender<T>
where
    T: 'static + Serialize + for<'de> Deserialize<'de> + Send,
{
    pub async fn send(&mut self, value: T) -> Result<()> {
        match self {
            Self::Zenoh(tx) => tx.send(&value).await?,
            Self::RustDds(tx) => tx.send(value).await?,
            Sender::CycloneDds(tx) => tx.send(value).await?,
        }
        Ok(())
    }
}
