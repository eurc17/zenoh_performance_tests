use crate::{common::*, sender::Sender, state::State, stream::Event};
use zenoh as zn;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Defines the structure of the config file.
pub struct Config {
    /// The maximum number of rounds to run the reliable broadcast.
    pub max_rounds: usize,
    /// The number of extra rounds to send echo(m,s). It will not exceed the `max_rounds`
    pub extra_rounds: usize,
    /// The timeout for each round.
    #[serde(with = "humantime_serde")]
    pub round_timeout: Duration,
    /// The interval that publishes echo messages.
    #[serde(with = "humantime_serde")]
    pub echo_interval: Duration,
    /// I/O configuration.
    pub io: IoConfig,
}

impl Config {
    pub async fn build<T>(
        &self,
        zenoh_session: Option<Arc<zenoh::Session>>,
        dds_domain_participant: Option<&rustdds::DomainParticipant>,
    ) -> Result<
        (
            Sender<T>,
            impl Stream<Item = Result<Event<T>, Error>> + Send,
        ),
        Error,
    >
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync + Clone,
    {
        State::new(self, zenoh_session, dds_domain_participant).await
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IoConfig {
    Zenoh(zenoh_config::ZenohConfig),
    RustDds(crate::io::rustdds::Config),
    CycloneDds(crate::io::cyclonedds::Config),
}

impl From<crate::io::cyclonedds::Config> for IoConfig {
    fn from(v: crate::io::cyclonedds::Config) -> Self {
        Self::CycloneDds(v)
    }
}

impl From<crate::io::rustdds::Config> for IoConfig {
    fn from(v: crate::io::rustdds::Config) -> Self {
        Self::RustDds(v)
    }
}

impl From<zenoh_config::ZenohConfig> for IoConfig {
    fn from(v: zenoh_config::ZenohConfig) -> Self {
        Self::Zenoh(v)
    }
}

impl IoConfig {
    /// Returns `true` if the io config is [`Zenoh`].
    ///
    /// [`Zenoh`]: IoConfig::Zenoh
    pub fn is_zenoh(&self) -> bool {
        matches!(self, Self::Zenoh(..))
    }

    /// Returns `true` if the io config is [`RustDds`].
    ///
    /// [`RustDds`]: IoConfig::RustDds
    pub fn is_rust_dds(&self) -> bool {
        matches!(self, Self::RustDds(..))
    }

    /// Returns `true` if the io config is [`CycloneDds`].
    ///
    /// [`CycloneDds`]: IoConfig::CycloneDds
    pub fn is_cyclone_dds(&self) -> bool {
        matches!(self, Self::CycloneDds(..))
    }
}

pub mod zenoh_config {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub struct ZenohConfig {
        pub key: String,
        /// Subscription mode for Zenoh
        pub sub_mode: SubMode,
        /// Reliability QoS for Zenoh
        pub reliability: Reliability,
        /// Congestion control QoS for Zenoh
        pub congestion_control: CongestionControl,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SubMode {
        Push,
        Pull,
    }

    impl From<SubMode> for zn::subscriber::SubMode {
        fn from(from: SubMode) -> Self {
            use zn::subscriber::SubMode as O;
            use SubMode as I;

            match from {
                I::Push => O::Push,
                I::Pull => O::Pull,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Reliability {
        BestEffort,
        Reliable,
    }

    impl From<Reliability> for zn::subscriber::Reliability {
        fn from(from: Reliability) -> Self {
            use zn::subscriber::Reliability as O;
            use Reliability as I;

            match from {
                I::BestEffort => O::BestEffort,
                I::Reliable => O::Reliable,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CongestionControl {
        Block,
        Drop,
    }

    impl From<CongestionControl> for zn::publication::CongestionControl {
        fn from(from: CongestionControl) -> Self {
            use zn::publication::CongestionControl as O;
            use CongestionControl as I;

            match from {
                I::Block => O::Block,
                I::Drop => O::Drop,
            }
        }
    }
}
