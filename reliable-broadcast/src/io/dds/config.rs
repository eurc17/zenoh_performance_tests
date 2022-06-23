use super::{Receiver, Sender};
use anyhow::Result;
use rustdds::{
    policy::{Durability, History, Reliability},
    DomainParticipant, QosPolicies, QosPolicyBuilder, TopicKind, GUID,
};
use serde::{Deserialize, Serialize};
use std::any;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub topic: String,
    pub durability: Durability,
    pub reliability: Reliability,
    pub history: History,
}

impl Config {
    pub fn build_sender<T>(&self, domain_participant: &DomainParticipant) -> Result<Sender<T>>
    where
        T: Serialize,
    {
        let qos = self.qos();
        let topic = domain_participant.create_topic(
            self.topic.clone(),
            any::type_name::<T>().to_string(),
            &qos,
            TopicKind::WithKey,
        )?;
        let publisher = domain_participant.create_publisher(&qos)?;
        let writer = publisher.create_datawriter_cdr(&topic, Some(qos))?;

        Ok(Sender {
            writer: Some(writer),
            // key: get_key_for_topic(&self.topic),
            key: GUID::new_participant_guid(),
        })
    }

    pub fn build_receiver<T>(&self, domain_participant: &DomainParticipant) -> Result<Receiver<T>>
    where
        T: 'static + for<'de> Deserialize<'de>,
    {
        let qos = self.qos();
        let topic = domain_participant.create_topic(
            self.topic.clone(),
            any::type_name::<T>().to_string(),
            &qos,
            TopicKind::WithKey,
        )?;
        let subscriber = domain_participant.create_subscriber(&qos)?;
        let reader = subscriber.create_datareader_cdr(&topic, None)?;

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
