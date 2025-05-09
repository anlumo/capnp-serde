use capnp::{
    dynamic_value,
    introspect::{Introspect, TypeVariant},
    message::TypedBuilder,
    traits::Owned,
};
use serde::de::DeserializeSeed;
use tracing::trace;

use crate::types::{seq::SeqVisitor, structs::StructVisitor};

/// A deserialize implementation that can be used to deserialize data encoded in a serde format into a [`TypedBuilder`].
///
/// This can be used to convert from any format that implements serde, such as JSON, YAML or CBOR, into a Cap'n Proto message (based on a Cap'n Proto schema).
///
/// # Example
///
/// ```ignore
/// use capnp_serde::CapnpSerdeBuilder;
///
/// let value = CapnpSerdeBuilder::<my_type::Owned>::deserialize(&serde_json::json!(...)).unwrap();
/// let reader = capnp::message::TypedBuilder::from(value).get_root_as_reader().unwrap();
/// ```
pub struct CapnpSerdeBuilder<O: Owned> {
    message: capnp::message::TypedBuilder<O>,
}

impl<O: Owned> From<CapnpSerdeBuilder<O>> for TypedBuilder<O> {
    fn from(builder: CapnpSerdeBuilder<O>) -> Self {
        builder.message
    }
}

impl<'de, O> serde::de::Deserialize<'de> for CapnpSerdeBuilder<O>
where
    O: Owned + Introspect + 'static,
    for<'a> O::Builder<'a>: Into<capnp::dynamic_value::Builder<'a>>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        trace!(
            "CapnpSerdeBuilder<{}>::deserialize",
            std::any::type_name::<O>()
        );
        let mut instance = Self {
            message: TypedBuilder::<O>::new_default(),
        };
        {
            let ty = O::introspect();
            match ty.which() {
                TypeVariant::Struct(_) => {
                    let builder = instance.message.init_root();
                    let seed = StructVisitor {
                        builder: builder.into(),
                        ty,
                    };
                    seed.deserialize(deserializer)
                        .inspect_err(|err| tracing::error!("{err}"))?;
                }
                TypeVariant::List(inner_ty) => {
                    let seed = SeqVisitor::new(inner_ty, |size| {
                        let root = instance.message.initn_root(size).into();
                        if let dynamic_value::Builder::List(list) = root {
                            Ok(list)
                        } else {
                            Err(capnp::Error::failed("Not a list".to_owned()))
                        }
                    });
                    seed.deserialize(deserializer)
                        .inspect_err(|err| tracing::error!("{err}"))?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(instance)
    }
}

impl<O: Owned> AsRef<capnp::message::TypedBuilder<O>> for CapnpSerdeBuilder<O> {
    fn as_ref(&self) -> &capnp::message::TypedBuilder<O> {
        &self.message
    }
}

impl<O: Owned> AsMut<capnp::message::TypedBuilder<O>> for CapnpSerdeBuilder<O> {
    fn as_mut(&mut self) -> &mut capnp::message::TypedBuilder<O> {
        &mut self.message
    }
}
