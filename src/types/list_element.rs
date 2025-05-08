use capnp::{
    dynamic_value::{self, Enum},
    introspect::TypeVariant,
    schema::EnumSchema,
};
use serde::de::DeserializeSeed;
use tracing::{error, trace};

use crate::types::{STRUCT_ENUM_SCHEMA_FIELD_NAMES, enums::EnumVisitor};

use super::{
    bools::BoolVisitor, data::DataVisitor, num::NumVisitor, seq::SeqVisitor,
    structs::StructVisitor, text::TextVisitor, void::VoidVisitor,
};

pub(super) struct ElementSeed<'a> {
    pub(super) list_builder: capnp::dynamic_list::Builder<'a>,
    pub(super) index: u32,
    pub(super) ty: capnp::introspect::Type,
}

impl<'a, 'de> DeserializeSeed<'de> for &mut ElementSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        trace!("CapnpSerdeElementSeed::deserialize {:?}", self.ty);
        match self.ty.which() {
            TypeVariant::List(inner_ty) => {
                let list_builder = self.list_builder.reborrow();
                let seed = SeqVisitor {
                    inner_ty,
                    generator: |size| -> capnp::Result<capnp::dynamic_list::Builder<'_>> {
                        let builder = list_builder.init(self.index, size)?;
                        if let capnp::dynamic_value::Builder::List(list_builder) = builder {
                            Ok(list_builder)
                        } else {
                            Err(capnp::Error::failed("Internal error".to_owned()))
                        }
                    },
                };
                seed.deserialize(deserializer)
                    .inspect_err(|err| error!("{err}"))?;
            }
            TypeVariant::Text => {
                let list_builder = self.list_builder.reborrow();
                deserializer
                    .deserialize_str(TextVisitor::new(|s: &str| -> capnp::Result<()> {
                        let dynamic_value::Builder::Text(mut text_builder) =
                            list_builder.init(self.index, s.len() as _)?
                        else {
                            return Err(capnp::Error::failed("Internal error".to_owned()));
                        };

                        text_builder.push_str(s);
                        Ok(())
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Data => {
                deserializer
                    .deserialize_bytes(DataVisitor::new(|s: &[u8]| -> capnp::Result<()> {
                        self.list_builder
                            .set(self.index, dynamic_value::Reader::Data(s))?;
                        Ok(())
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Bool => {
                deserializer
                    .deserialize_bool(BoolVisitor::new(|b| {
                        self.list_builder
                            .set(self.index, dynamic_value::Reader::Bool(b))
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Int8 => {
                deserializer
                    .deserialize_i8(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Int8(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Int16 => {
                deserializer
                    .deserialize_i16(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Int16(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Int32 => {
                deserializer
                    .deserialize_i32(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Int32(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Int64 => {
                deserializer
                    .deserialize_i64(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Int64(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::UInt8 => {
                deserializer
                    .deserialize_u8(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::UInt8(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::UInt16 => {
                deserializer
                    .deserialize_u16(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::UInt16(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::UInt32 => {
                deserializer
                    .deserialize_u32(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::UInt32(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::UInt64 => {
                deserializer
                    .deserialize_u64(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::UInt64(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Float32 => {
                deserializer
                    .deserialize_f32(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Float32(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Float64 => {
                deserializer
                    .deserialize_f64(NumVisitor::new(|num| {
                        if let Some(num) = num {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Float64(num))
                        } else {
                            Ok(())
                        }
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Struct(_) => {
                let seed = StructVisitor {
                    builder: self
                        .list_builder
                        .reborrow()
                        .get(self.index)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?,
                    ty: self.ty,
                };
                seed.deserialize(deserializer)
                    .inspect_err(|err| error!("{err}"))?;
            }
            TypeVariant::Void => {
                deserializer
                    .deserialize_unit(VoidVisitor::new(|| {
                        self.list_builder
                            .set(self.index, dynamic_value::Reader::Void)
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Enum(raw_schema) => {
                let schema = EnumSchema::new(raw_schema);
                let proto = schema.get_proto();
                let enumerant_names = STRUCT_ENUM_SCHEMA_FIELD_NAMES.insert(proto.get_id(), |_| {
                    let enumerants = schema.get_enumerants().unwrap();
                    enumerants
                        .iter()
                        .map(|enumerant| {
                            enumerant.get_proto().get_name().unwrap().to_str().unwrap()
                        })
                        .collect::<Box<_>>()
                });

                deserializer
                    .deserialize_enum(
                        proto.get_display_name().unwrap().to_str().unwrap(),
                        enumerant_names,
                        EnumVisitor::new(schema, |enumerant| {
                            self.list_builder.set(
                                self.index,
                                dynamic_value::Reader::Enum(Enum::new(
                                    enumerant.get_ordinal(),
                                    schema,
                                )),
                            )
                        }),
                    )
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::AnyPointer => unimplemented!(),
            TypeVariant::Capability => unimplemented!(),
        }

        Ok(())
    }
}
