use crate::common::*;
use serde_json::Value;
use std::marker::PhantomData;
use zenoh::{
    prelude::{KeyExpr, SampleKind},
    publication::CongestionControl,
    Session,
};

#[derive(Debug, Clone, Default)]
pub struct SenderConfig {
    pub congestion_control: CongestionControl,
    pub kind: SampleKind,
}

impl SenderConfig {
    pub async fn build<'a, T, K>(self, session: Arc<Session>, key_expr: K) -> Result<Sender<T>>
    where
        K: Into<KeyExpr<'a>>,
        T: Serialize,
    {
        let Self {
            congestion_control,
            kind,
        } = self;
        Ok(Sender {
            session,
            key: key_expr.into().to_owned(),
            congestion_control,
            kind,
            _phantom: PhantomData,
        })
    }
}

pub struct Sender<T>
where
    T: Serialize,
{
    session: Arc<Session>,
    key: KeyExpr<'static>,
    congestion_control: CongestionControl,
    kind: SampleKind,
    _phantom: PhantomData<T>,
}

impl<T> Sender<T>
where
    T: Serialize,
{
    pub async fn send(&self, value: &T) -> Result<()> {
        let value: Value = serde_json::to_value(&value)?;
        self.session
            .put(&self.key, value)
            .congestion_control(self.congestion_control)
            .kind(self.kind)
            .await?;
        Ok(())
    }
}
