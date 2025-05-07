use capnp::{
    dynamic_value,
    introspect::{Introspect, TypeVariant},
    message::TypedBuilder,
    traits::Owned,
};
use serde::de::DeserializeSeed;
use tracing::trace;

use crate::types::{seq::SeqVisitor, structs::StructVisitor};

pub struct CapnpSerdeBuilder<O: Owned> {
    message: capnp::message::TypedBuilder<O>,
}

impl<O: Owned> CapnpSerdeBuilder<O> {
    pub fn into_inner(self) -> TypedBuilder<O> {
        self.message
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
                    seed.deserialize(deserializer)?;
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
                    seed.deserialize(deserializer)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(instance)
    }
}
