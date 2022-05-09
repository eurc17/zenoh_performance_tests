use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct Msg(pub Arc<[u8]>);

impl AsRef<[u8]> for Msg {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Serialize for Msg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Msg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = Vec::<u8>::deserialize(deserializer)?;
        Ok(Self(payload.into()))
    }
}
