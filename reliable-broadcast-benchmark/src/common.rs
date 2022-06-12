pub use futures::{
    future::{self, FutureExt},
    stream,
    stream::TryStreamExt,
    Stream, StreamExt,
};
pub use log::debug;
pub use serde::{Deserialize, Serialize};
pub use std::{
    sync::Arc,
    time::{Duration, Instant},
};
pub use zenoh as zn;

use std::error::Error as StdError;

pub type Error = Box<dyn StdError + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(non_snake_case)]
pub fn ZOk<T>(value: T) -> Result<T> {
    Ok(value)
}
