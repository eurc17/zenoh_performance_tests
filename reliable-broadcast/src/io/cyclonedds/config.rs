use super::{Receiver, Sender};
use anyhow::Result;
use cyclonedds_rs::{
    dds_participant::DdsParticipant,
    dds_qos::{dds_durability_kind, dds_history_kind, dds_reliability_kind},
    DdsPublisher, DdsQos, DdsReader, DdsSubscriber, DdsTopic, DdsWriter,
};
use rustdds::policy::{Durability, History, Reliability};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub topic: String,
    pub durability: Durability,
    pub reliability: Reliability,
    pub history: History,
}

impl Config {
    pub fn build_sender<T>(&self) -> Result<Sender<T>>
    where
        T: 'static + Serialize + for<'de> Deserialize<'de>,
    {
        let qos = self.qos()?;
        let part = DdsParticipant::create(None, Some(qos.clone()), None)?;
        let topic = DdsTopic::create(&part, &self.topic, Some(qos.clone()), None)?;

        let publisher = DdsPublisher::create(&part, Some(qos.clone()), None)?;
        let writer = DdsWriter::create(&publisher, topic, Some(qos.clone()), None)?;

        let key = Uuid::new_v4();

        Ok(Sender {
            writer: Some(writer),
            key,
        })
    }

    pub fn build_receiver<T>(&self) -> Result<Receiver<T>>
    where
        T: 'static + Serialize + for<'de> Deserialize<'de>,
    {
        let qos = self.qos()?;
        let part = DdsParticipant::create(None, Some(qos.clone()), None)?;
        let topic = DdsTopic::create(&part, &self.topic, Some(qos.clone()), None)?;

        let subscriber = DdsSubscriber::create(&part, Some(qos.clone()), None)?;
        let reader = DdsReader::create(&subscriber, topic, Some(qos.clone()), None)?;

        Ok(Receiver {
            reader: Some(reader),
        })
    }

    pub fn qos(&self) -> Result<DdsQos> {
        let mut qos = DdsQos::create()?;
        qos.set_durability(self.durability());

        {
            let (kind, depth) = self.history();
            qos.set_history(kind, depth.map(|d| d as i32).unwrap_or(0));
        }

        {
            let (kind, dur) = self.reliability();
            qos.set_reliability(kind, dur.unwrap_or(Duration::ZERO));
        }
        Ok(qos)
    }

    fn durability(&self) -> dds_durability_kind {
        match self.durability {
            Durability::Volatile => dds_durability_kind::DDS_DURABILITY_VOLATILE,
            Durability::TransientLocal => dds_durability_kind::DDS_DURABILITY_TRANSIENT_LOCAL,
            Durability::Transient => dds_durability_kind::DDS_DURABILITY_TRANSIENT,
            Durability::Persistent => dds_durability_kind::DDS_DURABILITY_PERSISTENT,
        }
    }

    fn history(&self) -> (dds_history_kind, Option<i32>) {
        match self.history {
            History::KeepLast { depth } => (dds_history_kind::DDS_HISTORY_KEEP_LAST, Some(depth)),
            History::KeepAll => (dds_history_kind::DDS_HISTORY_KEEP_ALL, None),
        }
    }

    fn reliability(&self) -> (dds_reliability_kind, Option<Duration>) {
        match self.reliability {
            Reliability::BestEffort => (dds_reliability_kind::DDS_RELIABILITY_BEST_EFFORT, None),
            Reliability::Reliable { max_blocking_time } => (
                dds_reliability_kind::DDS_RELIABILITY_RELIABLE,
                Some(max_blocking_time.to_std()),
            ),
        }
    }
}
