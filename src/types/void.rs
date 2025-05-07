use serde::de::Visitor;

pub(super) struct VoidVisitor<F> {
    setter: F,
}

impl<F> VoidVisitor<F> {
    pub(super) fn new(setter: F) -> Self {
        Self { setter }
    }
}

impl<'de, F, R> Visitor<'de> for VoidVisitor<F>
where
    F: FnOnce() -> R,
{
    type Value = R;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "void")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok((self.setter)())
    }
}
