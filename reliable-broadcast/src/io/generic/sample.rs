use super::super::{cyclonedds as cdds_io, rustdds as rustdds_io};
use crate::common::*;

pub enum Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    Zenoh(zenoh::prelude::Sample),
    RustDds(rustdds::with_key::DataSample<rustdds_io::KeyedSample<T>>),
    CycloneDds(cdds_io::KeyedSample<T>),
}

impl<T> Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn timestamp(&self) -> Timestamp {
        use Timestamp as T;

        match self {
            Sample::Zenoh(sample) => {
                let ts = sample
                    .timestamp
                    .unwrap_or_else(|| panic!("HLC feature must be enabled for Zenoh"));
                T::Zenoh(ts)
            }
            Sample::RustDds(sample) => {
                let ts = sample.sample_info().source_timestamp().unwrap_or_else(|| {
                    panic!("unable to retrieve source timestamp from DDS publisher")
                });
                T::RustDds(ts)
            }
            Sample::CycloneDds(sample) => T::CycloneDds(sample.timestamp),
        }
    }

    pub fn to_value(&self) -> Result<Option<T>>
    where
        T: Clone,
    {
        let value = match self {
            Sample::Zenoh(sample) => {
                use zenoh::prelude::SampleKind;

                if sample.kind != SampleKind::Put {
                    return Ok(None);
                }

                guard!(let Some(value) = sample.value.as_json() else {
                    return Err(anyhow!("unable to decode message: not JSON format").into());
                });

                let value: T = serde_json::from_value(value)?;
                value
            }
            Sample::RustDds(sample) => {
                let value = match sample.value().as_ref().ok() {
                    Some(value) => value,
                    None => return Ok(None),
                };
                value.data.clone()
            }
            Sample::CycloneDds(sample) => sample.data.clone(),
        };

        Ok(Some(value))
    }
}

impl<T> From<cdds_io::KeyedSample<T>> for Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from(v: cdds_io::KeyedSample<T>) -> Self {
        Self::CycloneDds(v)
    }
}

impl<T> From<rustdds::with_key::DataSample<rustdds_io::KeyedSample<T>>> for Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from(v: rustdds::with_key::DataSample<rustdds_io::KeyedSample<T>>) -> Self {
        Self::RustDds(v)
    }
}

impl<T> From<zenoh::prelude::Sample> for Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from(v: zenoh::prelude::Sample) -> Self {
        Self::Zenoh(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timestamp {
    Zenoh(uhlc::Timestamp),
    RustDds(rustdds::Timestamp),
    CycloneDds(Duration),
}

// impl From<rustdds::Timestamp> for Timestamp {
//     fn from(v: rustdds::Timestamp) -> Self {
//         Self::RustDds(v)
//     }
// }

// impl From<uhlc::Timestamp> for Timestamp {
//     fn from(v: uhlc::Timestamp) -> Self {
//         Self::Zenoh(v)
//     }
// }
