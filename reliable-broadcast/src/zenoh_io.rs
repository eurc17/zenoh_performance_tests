use crate::common::*;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use zn::subscriber::{SampleReceiver, Subscriber};

#[derive(Debug, Clone, Default)]
pub struct ZnSenderConfig {
    pub congestion_control: CongestionControl,
    pub kind: SampleKind,
}

impl ZnSenderConfig {
    pub async fn build<'a, K>(self, session: Arc<zn::Session>, key_expr: K) -> Result<ZnSender>
    where
        K: Into<KeyExpr<'a>>,
    {
        let Self {
            congestion_control,
            kind,
        } = self;
        Ok(ZnSender {
            session,
            key: key_expr.into().to_owned(),
            congestion_control,
            kind,
        })
    }
}

pub struct ZnSender {
    session: Arc<zn::Session>,
    key: KeyExpr<'static>,
    congestion_control: CongestionControl,
    kind: SampleKind,
}

impl ZnSender {
    pub async fn send<T>(&self, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let value: Value = serde_json::to_value(&value)?.into();
        self.session
            .put(&self.key, value)
            .congestion_control(self.congestion_control)
            .kind(self.kind)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZnReceiverConfig {
    pub reliability: Reliability,
    pub sub_mode: SubMode,
}

impl ZnReceiverConfig {
    pub async fn build<'a, K>(self, session: &zn::Session, key_expr: K) -> Result<ZnReceiver<'_>>
    where
        K: Into<KeyExpr<'a>>,
    {
        let subscriber_builder = session.subscribe(key_expr);
        let subscriber = subscriber_builder
            .reliability(self.reliability)
            .mode(self.sub_mode)
            .await?;
        Ok(ZnReceiver { subscriber })
    }
}

pub struct ZnReceiver<'a> {
    subscriber: Subscriber<'a>,
}

impl<'a> ZnReceiver<'a> {
    pub async fn recv(&mut self) -> Result<Sample> {
        let sample = self.subscriber.receiver().recv_async().await?;
        Ok(sample)
    }

    pub fn into_stream(mut self) -> ZnStream {
        let receiver = self.subscriber.receiver().clone();
        ZnStream { receiver }
    }
}

#[pin_project]
#[derive(Clone)]
pub struct ZnStream {
    #[pin]
    receiver: SampleReceiver,
}

impl Stream for ZnStream {
    type Item = Sample;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().receiver.poll_next(cx)
    }
}
