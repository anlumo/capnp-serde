use capnp::schema::{EnumSchema, Enumerant};
use serde::de::{DeserializeSeed, Visitor};
use tracing::trace;

pub(super) struct EnumVisitor<F> {
    schema: EnumSchema,
    setter: F,
}

impl<F, Value> EnumVisitor<F>
where
    F: FnOnce(Enumerant) -> Value,
{
    pub(super) fn new(schema: EnumSchema, setter: F) -> Self {
        Self { schema, setter }
    }
}

impl<'de, F, Value> Visitor<'de> for EnumVisitor<F>
where
    F: FnOnce(Enumerant) -> Value,
{
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "enum")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        trace!("EnumVisitor::visit_enum");
        // Cap'n Proto doesn't support data attached to enum variants, so we can
        // ignore that part
        data.variant_seed(self)
            .inspect_err(|err| tracing::error!("{err}"))
            .map_err(serde::de::Error::custom)
            .map(|(enumerant, _)| enumerant)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("EnumVisitor::visit_str");
        let enumerant = self
            .schema
            .get_enumerants()
            .inspect_err(|err| tracing::error!("{err}"))
            .map_err(serde::de::Error::custom)?
            .iter()
            .find(|enumerant| enumerant.get_proto().get_name().unwrap().to_str().unwrap() == value)
            .ok_or_else(|| serde::de::Error::custom("Unknown enumerant"))?;

        Ok((self.setter)(enumerant))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("EnumVisitor::visit_str");
        let enumerant = self
            .schema
            .get_enumerants()
            .inspect_err(|err| tracing::error!("{err}"))
            .map_err(serde::de::Error::custom)?
            .iter()
            .find(|enumerant| enumerant.get_proto().get_name().unwrap().to_str().unwrap() == value)
            .ok_or_else(|| serde::de::Error::custom("Unknown enumerant"))?;

        Ok((self.setter)(enumerant))
    }
}

impl<'de, F, Value> DeserializeSeed<'de> for EnumVisitor<F>
where
    F: FnOnce(Enumerant) -> Value,
{
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_identifier(self)
            .inspect_err(|err| tracing::error!("{err}"))
    }
}
