//! A module that performs operations related to messages.

use crate::common::*;

/// The identifier for a broadcast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BroadcastId {
    /// The ID of the message sender of (m, s).
    pub broadcaster: Uuid,
    /// The sequence number of the message.
    pub seq: usize,
}

impl Display for BroadcastId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.broadcaster, self.seq)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The enumeration of all types of messages used in reliable broadcast.
pub enum Message<T> {
    /// The message type of *(m, s)*.
    Broadcast(Broadcast<T>),
    /// The message type of *present*.
    Present(Present),
    /// The message type of *echo(m, s)*
    Echo(Echo),
}

impl<T> From<Broadcast<T>> for Message<T> {
    fn from(from: Broadcast<T>) -> Self {
        Self::Broadcast(from)
    }
}

impl<T> From<Present> for Message<T> {
    fn from(from: Present) -> Self {
        Self::Present(from)
    }
}

impl<T> From<Echo> for Message<T> {
    fn from(from: Echo) -> Self {
        Self::Echo(from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Derivative, Serialize, Deserialize)]
/// The structure for the message type *(m, s)*.
pub struct Broadcast<T> {
    pub from: Uuid,
    pub seq: usize,
    /// The data to send in the message.
    pub data: T,
}

impl<T> Broadcast<T> {
    pub fn broadcast_id(&self) -> BroadcastId {
        let Self { from, seq, .. } = *self;
        BroadcastId {
            broadcaster: from,
            seq,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Derivative, Serialize, Deserialize)]
/// The structure for the message type *echo(m, s)*
pub struct Echo {
    pub from: Uuid,
    pub broadcast_ids: Vec<BroadcastId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Derivative, Serialize, Deserialize)]
/// The structure for the message type *present*.
pub struct Present {
    pub from: Uuid,
}
