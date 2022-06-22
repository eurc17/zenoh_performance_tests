use super::super::dds as dds_io;
use crate::common::*;

pub enum Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    Zenoh(zenoh::prelude::Sample),
    Dds(rustdds::with_key::DataSample<dds_io::KeyedSample<T>>),
}

impl<T> Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn timestamp(&self) -> uhlc::Timestamp {
        match self {
            Sample::Zenoh(sample) => sample
                .timestamp
                .unwrap_or_else(|| panic!("HLC feature must be enabled for Zenoh")),
            Sample::Dds(sample) => todo!(),
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
            Sample::Dds(sample) => {
                let value = match sample.value().as_ref().ok() {
                    Some(value) => value,
                    None => return Ok(None),
                };
                value.data.clone()
            }
        };

        Ok(Some(value))
    }
}

impl<T> From<rustdds::with_key::DataSample<dds_io::KeyedSample<T>>> for Sample<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from(v: rustdds::with_key::DataSample<dds_io::KeyedSample<T>>) -> Self {
        Self::Dds(v)
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
