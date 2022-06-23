mod config;
pub use config::*;

mod sample;
pub use sample::*;

mod sender;
pub use sender::*;

mod receiver;
pub use receiver::*;

#[cfg(test)]
mod tests {
    use crate::common::*;
    use anyhow::ensure;
    use async_std::task::spawn;
    use rustdds::{
        policy::{Durability, History, Reliability},
        DomainParticipant, Duration,
    };

    use super::*;

    #[async_std::test]
    pub async fn dds_test() -> Result<()> {
        type Msg = u64;
        const NUM_VALUES: u64 = 100;

        let config = Config {
            topic: "mytopic".into(),
            durability: Durability::Persistent,
            reliability: Reliability::Reliable {
                max_blocking_time: Duration::from_millis(100),
            },
            history: History::KeepLast { depth: 1 },
        };
        let domain_participant = DomainParticipant::new(0)?;
        let mut tx = config.build_sender::<Msg>(&domain_participant)?;
        let mut rx = config.build_receiver::<Msg>(&domain_participant)?;

        let pub_task = spawn(async move {
            for val in 0..NUM_VALUES {
                tx.send(val).await?;
            }
            anyhow::Ok(())
        });

        let sub_task = spawn(async move {
            for expect in 0..NUM_VALUES {
                let val = rx.recv().await?;
                let val = match val {
                    Some(val) => val,
                    None => break,
                };
                ensure!(val == expect, "expect {}, but received {}", expect, val);
            }
            anyhow::Ok(())
        });

        future::try_join(pub_task, sub_task).await?;

        Ok(())
    }
}
