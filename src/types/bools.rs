use serde::de::Visitor;

pub(super) struct BoolVisitor<F> {
    setter: F,
}

impl<F> BoolVisitor<F> {
    pub(super) fn new(setter: F) -> Self {
        Self { setter }
    }
}

impl<'de, F, R> Visitor<'de> for BoolVisitor<F>
where
    F: FnOnce(bool) -> R,
{
    type Value = R;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bool")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok((self.setter)(v))
    }
}
