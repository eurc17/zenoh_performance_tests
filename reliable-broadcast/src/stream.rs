use crate::{common::*, message::BroadcastId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event<T> {
    pub result: Result<T, ConsensusError>,
    pub broadcast_id: BroadcastId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsensusError {
    /// Error for the number of peers is less than 4.
    InsufficientPeers,
    /// Error for the number of echos is less than 1/3 for peers.
    InsufficientEchos,
    /// Error for the number of echos is less than 2/3 for peers.
    ConsensusLost,
}
