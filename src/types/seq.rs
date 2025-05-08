use capnp::introspect::TypeVariant;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use tracing::{error, trace};

use crate::types::enums::EnumVisitor;

use super::{list_element::ElementSeed, type_variant_to_str};

// Sequences only know their length at deserialization time, so we have to delay
// the initialization of the field
pub(crate) struct SeqVisitor<F> {
    pub(super) inner_ty: capnp::introspect::Type,
    pub(super) generator: F,
}

impl<'a, F> SeqVisitor<F>
where
    F: FnOnce(u32) -> capnp::Result<capnp::dynamic_list::Builder<'a>>,
{
    pub(crate) fn new(inner_ty: capnp::introspect::Type, generator: F) -> Self {
        Self {
            inner_ty,
            generator,
        }
    }
}

impl<'a, 'de, F> Visitor<'de> for SeqVisitor<F>
where
    F: FnOnce(u32) -> capnp::Result<capnp::dynamic_list::Builder<'a>>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let TypeVariant::List(inner_ty) = self.inner_ty.which() else {
            return Err(std::fmt::Error);
        };
        write!(formatter, "List({})", type_variant_to_str(inner_ty.which()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        trace!(
            "CapnpSerdeSeqVisitor::visit_seq size = {:?}",
            seq.size_hint()
        );
        if let Some(size) = seq.size_hint() {
            // The try-operator confuses the borrow checker here, so we can't use it!
            let list_builder = match (self.generator)(size as u32) {
                Ok(list_builder) => list_builder,
                Err(err) => {
                    error!("{err}");
                    return Err(serde::de::Error::custom(err));
                }
            };
            let mut index = 0;
            let mut seed = ElementSeed {
                list_builder,
                index: 0,
                ty: self.inner_ty,
            };
            loop {
                seed.index = index;
                match seq.next_element_seed(&mut seed) {
                    Err(err) => {
                        error!("{err}");
                        return Err(err);
                    }
                    Ok(None) => break,
                    Ok(Some(())) => {
                        index += 1;
                    }
                }
            }
            Ok(())
        } else {
            match self.inner_ty.which() {
                TypeVariant::Void => {
                    let mut count = 0;
                    while seq.next_element::<()>()?.is_some() {
                        count += 1;
                    }
                    (self.generator)(count)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)
                        .map(|_| ())
                }
                TypeVariant::Bool => iterate_simple::<'_, '_, bool, _, _>(self.generator, seq),
                TypeVariant::Int8 => iterate_simple::<'_, '_, i8, _, _>(self.generator, seq),
                TypeVariant::Int16 => iterate_simple::<'_, '_, i16, _, _>(self.generator, seq),
                TypeVariant::Int32 => iterate_simple::<'_, '_, i32, _, _>(self.generator, seq),
                TypeVariant::Int64 => iterate_simple::<'_, '_, i64, _, _>(self.generator, seq),
                TypeVariant::UInt8 => iterate_simple::<'_, '_, u8, _, _>(self.generator, seq),
                TypeVariant::UInt16 => iterate_simple::<'_, '_, u16, _, _>(self.generator, seq),
                TypeVariant::UInt32 => iterate_simple::<'_, '_, u32, _, _>(self.generator, seq),
                TypeVariant::UInt64 => iterate_simple::<'_, '_, u64, _, _>(self.generator, seq),
                TypeVariant::Float32 => iterate_simple::<'_, '_, f32, _, _>(self.generator, seq),
                TypeVariant::Float64 => iterate_simple::<'_, '_, f64, _, _>(self.generator, seq),
                TypeVariant::Text => {
                    let mut values = Vec::new();
                    while let Some(value) = seq.next_element::<String>()? {
                        values.push(value);
                    }
                    let mut list_builder = (self.generator)(values.len() as _)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?;
                    for (index, value) in values.into_iter().enumerate() {
                        let capnp::dynamic_value::Builder::Text(mut text_builder) = list_builder
                            .reborrow()
                            .init(index as u32, value.len() as u32)
                            .inspect_err(|err| error!("{err}"))
                            .map_err(serde::de::Error::custom)?
                        else {
                            return Err(serde::de::Error::custom("Internal error".to_owned()));
                        };
                        text_builder.push_str(&value);
                    }
                    Ok(())
                }
                TypeVariant::Data => {
                    let mut values = Vec::new();
                    while let Some(value) = seq.next_element::<serde_bytes::ByteBuf>()? {
                        values.push(value);
                    }
                    let mut list_builder = (self.generator)(values.len() as _)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?;
                    for (index, value) in values.into_iter().enumerate() {
                        list_builder
                            .reborrow()
                            .set(
                                index as u32,
                                capnp::dynamic_value::Reader::Data(value.as_ref()),
                            )
                            .inspect_err(|err| error!("{err}"))
                            .map_err(serde::de::Error::custom)?
                    }
                    Ok(())
                }
                TypeVariant::Enum(raw_enum_schema) => {
                    let mut values = Vec::new();
                    while seq
                        .next_element_seed(EnumVisitor::new(raw_enum_schema.into(), |value| {
                            values.push(value);
                        }))?
                        .is_some()
                    {}
                    let mut list_builder = (self.generator)(values.len() as _)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?;
                    for (index, value) in values.into_iter().enumerate() {
                        list_builder
                            .reborrow()
                            .set(
                                index as u32,
                                capnp::dynamic_value::Reader::Enum(
                                    capnp::dynamic_value::Enum::new(
                                        value.get_ordinal(),
                                        value.get_containing_enum(),
                                    ),
                                ),
                            )
                            .inspect_err(|err| error!("{err}"))
                            .map_err(serde::de::Error::custom)?
                    }
                    Ok(())
                }
                TypeVariant::Struct(_) | TypeVariant::List(_) => Err(serde::de::Error::custom(
                    "Cap'n Proto encoding requires pointer lists to declare their size before the actual data. Your decoder does not provide this information.",
                )),
                TypeVariant::AnyPointer => unimplemented!(),
                TypeVariant::Capability => unimplemented!(),
            }
        }
    }
}

impl<'a, 'de, F> DeserializeSeed<'de> for SeqVisitor<F>
where
    F: FnOnce(u32) -> capnp::Result<capnp::dynamic_list::Builder<'a>>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(self)
            .inspect_err(|err| error!("{err}"))?;
        Ok(())
    }
}

fn iterate_simple<'a, 'de, Value, F, A>(generator: F, mut seq: A) -> Result<(), A::Error>
where
    Value: serde::Deserialize<'de>,
    A: SeqAccess<'de>,
    for<'b> capnp::dynamic_value::Reader<'b>: From<Value>,
    F: FnOnce(u32) -> capnp::Result<capnp::dynamic_list::Builder<'a>>,
{
    let mut values = Vec::new();
    while let Some(value) = seq.next_element::<Value>()? {
        values.push(value);
    }
    let mut list_builder = generator(values.len() as _)
        .inspect_err(|err| error!("{err}"))
        .map_err(serde::de::Error::custom)?;
    for (index, value) in values.into_iter().enumerate() {
        list_builder
            .set(index as u32, value.into())
            .inspect_err(|err| error!("{err}"))
            .map_err(serde::de::Error::custom)?;
    }
    Ok(())
}
