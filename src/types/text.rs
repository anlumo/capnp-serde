use serde::de::Visitor;
use tracing::trace;

pub(super) struct TextVisitor<F> {
    setter: F,
}

impl<F> TextVisitor<F> {
    pub(super) fn new(setter: F) -> Self {
        Self { setter }
    }
}

impl<'de, F, Value> Visitor<'de> for TextVisitor<F>
where
    F: FnOnce(&str) -> Value,
{
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "text")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("TextVisitor::visit_str {v:?}");
        Ok((self.setter)(v))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("TextVisitor::visit_string {v:?}");
        Ok((self.setter)(&v))
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("TextVisitor::visit_borrowed_str {v:?}");
        Ok((self.setter)(v))
    }
}
