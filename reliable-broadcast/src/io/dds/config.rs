use super::{Receiver, Sender};
use crate::common::*;
use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rustdds::{
    policy::{Durability, History, Reliability},
    DomainParticipant, QosPolicies, QosPolicyBuilder, TopicKind,
};
use serde::{Deserialize, Serialize};
use std::any;

pub(crate) static GLOBAL_DOMAIN_PARTICIPANTS: Lazy<DashMap<u16, Arc<DomainParticipant>>> =
    Lazy::new(DashMap::new);

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_domain_id")]
    pub domain_id: u16,
    pub topic: String,
    pub durability: Durability,
    pub reliability: Reliability,
    pub history: History,
}

fn default_domain_id() -> u16 {
    0
}

impl Config {
    pub async fn build_sender<T>(&self) -> Result<Sender<T>>
    where
        T: Serialize,
    {
        let domain_participant = get_domain_participant(self.domain_id)?;
        let qos = self.qos();
        let topic = domain_participant.create_topic(
            self.topic.clone(),
            any::type_name::<T>().to_string(),
            &qos,
            TopicKind::WithKey,
        )?;
        let publisher = domain_participant.create_publisher(&qos)?;
        let writer = publisher.create_datawriter_no_key_cdr::<T>(&topic, Some(qos))?;

        Ok(Sender {
            writer: Some(writer),
        })
    }

    pub async fn build_receiver<T>(&self) -> Result<Receiver<T>>
    where
        T: 'static + for<'de> Deserialize<'de>,
    {
        let domain_participant = get_domain_participant(self.domain_id)?;
        let qos = self.qos();
        let topic = domain_participant.create_topic(
            self.topic.clone(),
            any::type_name::<T>().to_string(),
            &qos,
            TopicKind::WithKey,
        )?;
        let subscriber = domain_participant.create_subscriber(&qos)?;
        let reader = subscriber
            .create_datareader_no_key_cdr::<T>(&topic, None)
            .unwrap();

        Ok(Receiver {
            reader: Some(reader),
        })
    }

    pub fn qos(&self) -> QosPolicies {
        QosPolicyBuilder::new()
            .reliability(self.reliability)
            .durability(self.durability)
            .history(self.history)
            .build()
    }
}
