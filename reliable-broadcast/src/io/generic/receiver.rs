use super::{
    super::{dds as dds_io, zenoh as zenoh_io},
    Sample,
};
use crate::common::*;
use futures::stream::BoxStream;

pub enum Receiver<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    Zenoh(zenoh_io::Receiver<T>),
    Dds(dds_io::Receiver<T>),
}

impl<T> From<dds_io::Receiver<T>> for Receiver<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from(v: dds_io::Receiver<T>) -> Self {
        Self::Dds(v)
    }
}

impl<T> From<zenoh_io::Receiver<T>> for Receiver<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from(v: zenoh_io::Receiver<T>) -> Self {
        Self::Zenoh(v)
    }
}

impl<T> Receiver<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn into_sample_stream(self) -> BoxStream<'static, Result<Sample<T>>> {
        match self {
            Receiver::Zenoh(rx) => rx.into_sample_stream().map(Sample::from).map(Ok).boxed(),
            Receiver::Dds(rx) => rx
                .into_sample_stream()
                .map(|s| Ok(Sample::from(s?)))
                .boxed(),
        }
    }
}
