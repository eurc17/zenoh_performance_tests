use crate::common::*;
use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rustdds::{DomainParticipant, GUID};

static GLOBAL_DOMAIN_PARTICIPANTS: Lazy<DashMap<u16, Arc<DomainParticipant>>> =
    Lazy::new(DashMap::new);

static GLOBAL_TOPIC_KEY_MAP: Lazy<DashMap<String, GUID>> = Lazy::new(|| DashMap::new());

pub fn get_domain_participant(domain_id: u16) -> Result<Arc<DomainParticipant>> {
    let ref_ = GLOBAL_DOMAIN_PARTICIPANTS
        .entry(domain_id)
        .or_try_insert_with(|| -> Result<_> {
            let part = DomainParticipant::new(domain_id)?;
            Ok(Arc::new(part))
        })?;
    let participant = ref_.value().clone();
    Ok(participant)
}

pub fn get_key_for_topic(topic: &str) -> GUID {
    let entry = GLOBAL_TOPIC_KEY_MAP
        .entry(topic.to_string())
        .or_insert_with(|| GUID::new_participant_guid());
    *entry.value()
}
