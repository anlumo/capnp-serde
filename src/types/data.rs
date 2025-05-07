use serde::de::Visitor;
use tracing::trace;

pub(super) struct DataVisitor<F> {
    setter: F,
}

impl<F> DataVisitor<F> {
    pub(super) fn new(setter: F) -> Self {
        Self { setter }
    }
}

impl<'de, F, Value> Visitor<'de> for DataVisitor<F>
where
    F: FnOnce(&[u8]) -> Value,
{
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "text")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("DataVisitor::visit_bytes {v:?}");
        Ok((self.setter)(v))
    }
}
