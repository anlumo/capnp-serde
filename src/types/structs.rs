use capnp::{
    dynamic_value,
    introspect::TypeVariant,
    schema::{EnumSchema, StructSchema},
};
use serde::de::{DeserializeSeed, MapAccess, Unexpected, Visitor};
use tracing::{error, trace};

use crate::types::enums::EnumVisitor;

use super::{
    STRUCT_ENUM_SCHEMA_FIELD_NAMES, dynamic_value_type_to_str, seq::SeqVisitor, type_variant_to_str,
};

pub(crate) struct StructVisitor<'a> {
    pub(crate) builder: capnp::dynamic_value::Builder<'a>,
    pub(crate) ty: capnp::introspect::Type,
}

impl<'a, 'de> DeserializeSeed<'de> for StructVisitor<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        trace!("StructSeed::deserialize {:?}", self.ty);
        match self.ty.which() {
            TypeVariant::Void => deserializer
                .deserialize_unit(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Bool => deserializer
                .deserialize_bool(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Int8 => deserializer
                .deserialize_i8(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Int16 => deserializer
                .deserialize_i16(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Int32 => deserializer
                .deserialize_i32(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Int64 => deserializer
                .deserialize_i64(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::UInt8 => deserializer
                .deserialize_u8(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::UInt16 => deserializer
                .deserialize_u16(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::UInt32 => deserializer
                .deserialize_u32(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::UInt64 => deserializer
                .deserialize_u64(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Float32 => deserializer
                .deserialize_f32(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Float64 => deserializer
                .deserialize_f64(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Text => todo!(),
            TypeVariant::Data => todo!(),
            TypeVariant::Struct(raw_branded_struct_schema) => {
                let schema = StructSchema::new(raw_branded_struct_schema);
                let proto = schema.get_proto();
                let field_names = STRUCT_ENUM_SCHEMA_FIELD_NAMES.insert(proto.get_id(), |_| {
                    let fields = schema.get_fields().unwrap();
                    fields
                        .iter()
                        .map(|field| field.get_proto().get_name().unwrap().to_str().unwrap())
                        .collect::<Box<_>>()
                });

                trace!(
                    "deserialize struct {}, field names = {:?}",
                    proto.get_display_name().unwrap().to_str().unwrap(),
                    field_names
                );

                deserializer
                    .deserialize_struct(
                        proto.get_display_name().unwrap().to_str().unwrap(),
                        field_names,
                        self,
                    )
                    .inspect_err(|err| error!("{err}"))?;
            }
            TypeVariant::List(_) => deserializer
                .deserialize_seq(self)
                .inspect_err(|err| error!("{err}"))?,
            TypeVariant::Enum(_) => unimplemented!(),
            TypeVariant::AnyPointer => unimplemented!(),
            TypeVariant::Capability => unimplemented!(),
        }
        Ok(())
    }
}

impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", type_variant_to_str(self.ty.which()))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let capnp::dynamic_value::Builder::Struct(mut struct_builder) = self.builder else {
            return Err(serde::de::Error::invalid_type(
                Unexpected::Map,
                &dynamic_value_type_to_str(&self.builder),
            ));
        };
        let TypeVariant::Struct(raw_schema) = self.ty.which() else {
            return Err(serde::de::Error::invalid_type(
                Unexpected::Map,
                &type_variant_to_str(self.ty.which()),
            ));
        };
        let schema = StructSchema::new(raw_schema);
        trace!(
            "StructSeed::visit_map {:?}",
            schema
                .get_proto()
                .get_display_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        loop {
            trace!("StructSeed::visit_map loop calling next_key");
            let key = match map.next_key::<String>() {
                Err(err) => {
                    error!("{err}");
                    return Err(err);
                }
                Ok(None) => break,
                Ok(Some(key)) => key,
            };
            let field = match schema.get_field_by_name(&key) {
                Ok(field) => field,
                Err(err) => {
                    error!("{err}");
                    return Err(serde::de::Error::custom(err));
                }
            };
            trace!(
                "StructSeed::visit_map key = {key:?}, type = {:?}",
                field.get_type()
            );
            match field.get_type().which() {
                TypeVariant::List(inner_ty) => {
                    let struct_builder = struct_builder.reborrow();
                    map.next_value_seed(SeqVisitor {
                        inner_ty,
                        generator: |size| {
                            let builder = struct_builder
                                .initn(field, size)
                                .inspect_err(|err| error!("{err}"))?;
                            if let capnp::dynamic_value::Builder::List(list_builder) = builder {
                                Ok(list_builder)
                            } else {
                                Err(capnp::Error::failed("Internal error".to_owned()))
                            }
                        },
                    })?
                }
                TypeVariant::Text => {
                    let text: String = map.next_value()?;
                    let dynamic_value::Builder::Text(mut text_builder) = struct_builder
                        .reborrow()
                        .initn(field, text.len() as u32)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?
                    else {
                        return Err(serde::de::Error::custom("Internal error"));
                    };
                    text_builder.push_str(&text);
                }
                TypeVariant::Data => {
                    let bytes: serde_bytes::ByteBuf =
                        map.next_value().inspect_err(|err| error!("{err}"))?;
                    if let Err(err) = struct_builder.set(field, dynamic_value::Reader::Data(&bytes))
                    {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Void => {
                    map.next_value::<()>()?; // ignore value
                }
                TypeVariant::Bool => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Bool(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int8 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Int8(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int16 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Int16(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int32 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Int32(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int64 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Int64(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt8 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::UInt8(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt16 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::UInt16(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt32 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::UInt32(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt64 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::UInt64(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Float32 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Float32(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Float64 => {
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Float64(
                            map.next_value().inspect_err(|err| error!("{err}"))?,
                        ),
                    ) {
                        error!("{err}");
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Enum(raw_schema) => {
                    let schema = EnumSchema::new(raw_schema);
                    map.next_value_seed(EnumVisitor::new(schema, |enumerant| {
                        struct_builder.set(
                            field,
                            dynamic_value::Reader::Enum(capnp::dynamic_value::Enum::new(
                                enumerant.get_ordinal(),
                                schema,
                            )),
                        )
                    }))
                    .inspect_err(|err| error!("{err}"))?
                    .inspect_err(|err| error!("{err}"))
                    .map_err(serde::de::Error::custom)?;
                }
                TypeVariant::Struct(_) => {
                    let builder = struct_builder
                        .reborrow()
                        .init(field)
                        .inspect_err(|err| error!("{err}"))
                        .map_err(serde::de::Error::custom)?;
                    let seed = StructVisitor {
                        builder,
                        ty: field.get_type(),
                    };
                    map.next_value_seed(seed)?;
                }
                TypeVariant::AnyPointer => unimplemented!(),
                TypeVariant::Capability => unimplemented!(),
            }
        }
        Ok(())
    }
}
