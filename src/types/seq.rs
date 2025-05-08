use capnp::introspect::TypeVariant;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use tracing::{error, trace};

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
        } else {
            todo!()
        }
        Ok(())
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
