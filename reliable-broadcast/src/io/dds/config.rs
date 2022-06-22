use super::{
    global::{get_domain_participant, get_key_for_topic},
    Receiver, Sender,
};
use anyhow::Result;
use rustdds::{
    policy::{Durability, History, Reliability},
    QosPolicies, QosPolicyBuilder, TopicKind,
};
use serde::{Deserialize, Serialize};
use std::any;

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
    pub fn build_sender<T>(&self) -> Result<Sender<T>>
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
        let writer = publisher.create_datawriter_cdr(&topic, Some(qos))?;

        Ok(Sender {
            writer: Some(writer),
            key: get_key_for_topic(&self.topic),
        })
    }

    pub fn build_receiver<T>(&self) -> Result<Receiver<T>>
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
