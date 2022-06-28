use std::time::Duration;

use cyclonedds_rs::{cdr, TopicType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyedSample<T> {
    pub key: Uuid,
    #[serde(with = "serde_timestamp")]
    pub timestamp: Duration,
    pub data: T,
}

impl<T> TopicType for KeyedSample<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn has_key() -> bool {
        true
    }

    fn key_cdr(&self) -> Vec<u8> {
        cdr::serialize::<_, _, cdr::CdrLe>(&self.key, cdr::Infinite).unwrap()
    }

    fn force_md5_keyhash() -> bool {
        false
    }
}

mod serde_timestamp {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(ts: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let nanos = ts.as_nanos() as u64;
        nanos.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos))
    }
}
