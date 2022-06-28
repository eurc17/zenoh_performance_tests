use super::{
    super::{cyclonedds as cdds_io, rustdds as rustdds_io, zenoh as zenoh_io},
    Sample,
};
use crate::common::*;
use futures::stream::BoxStream;

pub enum Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    Zenoh(zenoh_io::Receiver<T>),
    RustDds(rustdds_io::Receiver<T>),
    CycloneDds(cdds_io::Receiver<T>),
}

impl<T> From<cdds_io::Receiver<T>> for Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn from(v: cdds_io::Receiver<T>) -> Self {
        Self::CycloneDds(v)
    }
}

impl<T> From<rustdds_io::Receiver<T>> for Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn from(v: rustdds_io::Receiver<T>) -> Self {
        Self::RustDds(v)
    }
}

impl<T> From<zenoh_io::Receiver<T>> for Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn from(v: zenoh_io::Receiver<T>) -> Self {
        Self::Zenoh(v)
    }
}

impl<T> Receiver<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn into_sample_stream(self) -> BoxStream<'static, Result<Sample<T>>>
    where
        T: Sync + Clone,
    {
        match self {
            Receiver::Zenoh(rx) => rx.into_sample_stream().map(Sample::from).map(Ok).boxed(),
            Receiver::RustDds(rx) => rx
                .into_sample_stream()
                .map(|s| Ok(Sample::from(s?)))
                .boxed(),
            Receiver::CycloneDds(rx) => rx
                .into_sample_stream()
                .map(|s| Ok(Sample::from((*s?).clone())))
                .boxed(),
        }
    }
}
