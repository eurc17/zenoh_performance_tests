pub enum Sample<T> {
    Zenoh(zenoh::prelude::Sample),
    Dds(rustdds::no_key::DataSample<T>),
}

impl<T> From<rustdds::no_key::DataSample<T>> for Sample<T> {
    fn from(v: rustdds::no_key::DataSample<T>) -> Self {
        Self::Dds(v)
    }
}

impl<T> From<zenoh::prelude::Sample> for Sample<T> {
    fn from(v: zenoh::prelude::Sample) -> Self {
        Self::Zenoh(v)
    }
}
