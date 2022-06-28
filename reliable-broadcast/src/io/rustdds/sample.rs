use rustdds::{Keyed, GUID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyedSample<T> {
    pub key: GUID,
    pub data: T,
}

impl<T> Keyed for KeyedSample<T> {
    type K = GUID;

    fn key(&self) -> Self::K {
        self.key
    }
}
