use super::super::{dds as dds_io, zenoh as zenoh_io};
use crate::common::*;

pub enum Sender<T>
where
    T: Serialize,
{
    Zenoh(zenoh_io::Sender<T>),
    Dds(dds_io::Sender<T>),
}

impl<T> From<dds_io::Sender<T>> for Sender<T>
where
    T: Serialize,
{
    fn from(v: dds_io::Sender<T>) -> Self {
        Self::Dds(v)
    }
}

impl<T> From<zenoh_io::Sender<T>> for Sender<T>
where
    T: Serialize,
{
    fn from(v: zenoh_io::Sender<T>) -> Self {
        Self::Zenoh(v)
    }
}

impl<T> Sender<T>
where
    T: 'static + Serialize + Send,
{
    pub async fn send(&mut self, value: T) -> Result<()> {
        match self {
            Self::Zenoh(tx) => tx.send(&value).await?,
            Self::Dds(tx) => tx.send(value).await?,
        }
        Ok(())
    }
}
