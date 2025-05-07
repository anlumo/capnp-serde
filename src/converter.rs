use std::sync::LazyLock;

use capnp::{
    dynamic_value,
    introspect::{Introspect, TypeVariant},
    message::TypedBuilder,
    schema::{EnumSchema, StructSchema},
    traits::Owned,
};
use once_map::OnceMap;
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Unexpected, Visitor},
    ser::{Error as SerdeError, SerializeMap, SerializeSeq},
};
use tracing::trace;

#[repr(transparent)]
pub struct CapnpSerdeReader<'a>(dynamic_value::Reader<'a>);

impl<'a> CapnpSerdeReader<'a> {
    pub fn new(value: impl Into<dynamic_value::Reader<'a>>) -> Self {
        Self(value.into())
    }
}

impl serde::ser::Serialize for CapnpSerdeReader<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        trace!("CapnpSerdeReader::serialize {:?}", self.0);
        match self.0 {
            dynamic_value::Reader::Void => serializer.serialize_unit(),
            dynamic_value::Reader::Bool(value) => serializer.serialize_bool(value),
            dynamic_value::Reader::Int8(value) => serializer.serialize_i8(value),
            dynamic_value::Reader::Int16(value) => serializer.serialize_i16(value),
            dynamic_value::Reader::Int32(value) => serializer.serialize_i32(value),
            dynamic_value::Reader::Int64(value) => serializer.serialize_i64(value),
            dynamic_value::Reader::UInt8(value) => serializer.serialize_u8(value),
            dynamic_value::Reader::UInt16(value) => serializer.serialize_u16(value),
            dynamic_value::Reader::UInt32(value) => serializer.serialize_u32(value),
            dynamic_value::Reader::UInt64(value) => serializer.serialize_u64(value),
            dynamic_value::Reader::Float32(value) => serializer.serialize_f32(value),
            dynamic_value::Reader::Float64(value) => serializer.serialize_f64(value),
            dynamic_value::Reader::Enum(value) => {
                if let Some(enumerant) = value.get_enumerant().map_err(SerdeError::custom)? {
                    serializer.serialize_unit_variant(
                        enumerant
                            .get_containing_enum()
                            .get_proto()
                            .get_display_name()
                            .map_err(SerdeError::custom)?
                            .to_str()
                            .map_err(SerdeError::custom)?,
                        enumerant.get_ordinal() as _,
                        enumerant
                            .get_proto()
                            .get_name()
                            .map_err(SerdeError::custom)?
                            .to_str()
                            .map_err(SerdeError::custom)?,
                    )
                } else {
                    serializer.serialize_unit()
                }
            }
            dynamic_value::Reader::Text(reader) => {
                serializer.serialize_str(reader.to_str().map_err(SerdeError::custom)?)
            }
            dynamic_value::Reader::Data(items) => serializer.serialize_bytes(items),
            dynamic_value::Reader::Struct(reader) => {
                let fields = reader
                    .get_schema()
                    .get_fields()
                    .map_err(SerdeError::custom)?;
                let mut map = serializer.serialize_map(Some(fields.len() as _))?;
                for field in fields {
                    let name = field
                        .get_proto()
                        .get_name()
                        .map_err(SerdeError::custom)?
                        .to_str()
                        .map_err(SerdeError::custom)?;
                    map.serialize_entry(
                        name,
                        &Self(reader.get(field).map_err(SerdeError::custom)?),
                    )?;
                }
                map.end()
            }
            dynamic_value::Reader::List(reader) => {
                let mut sequence = serializer.serialize_seq(Some(reader.len() as _))?;
                for item in reader.iter() {
                    sequence.serialize_element(&Self(item.map_err(SerdeError::custom)?))?
                }
                sequence.end()
            }
            dynamic_value::Reader::AnyPointer(_) => {
                Err(SerdeError::custom("AnyPointer not supported"))
            }
            dynamic_value::Reader::Capability(_) => {
                Err(SerdeError::custom("Capability not supported"))
            }
        }
    }
}

// MARK: Deserializer

static STRUCT_ENUM_SCHEMA_FIELD_NAMES: LazyLock<OnceMap<u64, Box<[&'static str]>>> =
    LazyLock::new(OnceMap::new);

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
            let builder = instance.message.init_root();
            let ty = O::introspect();
            let seed = StructSeed {
                builder: builder.into(),
                ty,
            };
            seed.deserialize(deserializer)?;
        }
        Ok(instance)
    }
}

struct StructSeed<'a> {
    builder: capnp::dynamic_value::Builder<'a>,
    ty: capnp::introspect::Type,
}

impl<'a, 'de> DeserializeSeed<'de> for StructSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        trace!("StructSeed::deserialize {:?}", self.ty);
        match self.ty.which() {
            TypeVariant::Void => deserializer.deserialize_unit(self)?,
            TypeVariant::Bool => deserializer.deserialize_bool(self)?,
            TypeVariant::Int8 => deserializer.deserialize_i8(self)?,
            TypeVariant::Int16 => deserializer.deserialize_i16(self)?,
            TypeVariant::Int32 => deserializer.deserialize_i32(self)?,
            TypeVariant::Int64 => deserializer.deserialize_i64(self)?,
            TypeVariant::UInt8 => deserializer.deserialize_u8(self)?,
            TypeVariant::UInt16 => deserializer.deserialize_u16(self)?,
            TypeVariant::UInt32 => deserializer.deserialize_u32(self)?,
            TypeVariant::UInt64 => deserializer.deserialize_u64(self)?,
            TypeVariant::Float32 => deserializer.deserialize_f32(self)?,
            TypeVariant::Float64 => deserializer.deserialize_f64(self)?,
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

                deserializer.deserialize_struct(
                    proto.get_display_name().unwrap().to_str().unwrap(),
                    field_names,
                    self,
                )?;
            }
            TypeVariant::List(_) => deserializer.deserialize_seq(self)?,
            TypeVariant::Enum(raw_enum_schema) => {
                let schema = EnumSchema::new(raw_enum_schema);
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

                deserializer.deserialize_enum(
                    proto.get_display_name().unwrap().to_str().unwrap(),
                    enumerant_names,
                    self,
                )?
            }
            TypeVariant::AnyPointer => unimplemented!(),
            TypeVariant::Capability => unimplemented!(),
        }
        Ok(())
    }
}

impl<'a, 'de> Visitor<'de> for StructSeed<'a> {
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
            let key = match map.next_key::<String>() {
                Err(err) => return Err(err),
                Ok(None) => break,
                Ok(Some(key)) => key,
            };
            let field = match schema.get_field_by_name(&key) {
                Ok(field) => field,
                Err(err) => return Err(serde::de::Error::custom(err)),
            };
            trace!(
                "StructSeed::visit_map key = {key:?}, type = {:?}",
                field.get_type()
            );
            match field.get_type().which() {
                TypeVariant::List(inner_ty) => {
                    let struct_builder = struct_builder.reborrow();
                    map.next_value_seed(CapnpSerdeSeqVisitor {
                        inner_ty,
                        generator: |size| {
                            let builder = struct_builder.initn(field, size)?;
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
                        .map_err(serde::de::Error::custom)?
                    else {
                        return Err(serde::de::Error::custom("Internal error"));
                    };
                    text_builder.push_str(&text);
                }
                TypeVariant::Data => {
                    let bytes: serde_bytes::ByteBuf = map.next_value()?;
                    if let Err(err) = struct_builder.set(field, dynamic_value::Reader::Data(&bytes))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Void => {
                    map.next_value::<()>()?; // ignore value
                }
                TypeVariant::Bool => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Bool(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int8 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Int8(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int16 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Int16(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int32 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Int32(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Int64 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Int64(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt8 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::UInt8(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt16 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::UInt16(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt32 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::UInt32(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::UInt64 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::UInt64(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Float32 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Float32(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Float64 => {
                    if let Err(err) =
                        struct_builder.set(field, dynamic_value::Reader::Float64(map.next_value()?))
                    {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Enum(raw_schema) => {
                    let schema = EnumSchema::new(raw_schema);
                    if let Err(err) = struct_builder.set(
                        field,
                        dynamic_value::Reader::Enum(capnp::dynamic_value::Enum::new(
                            map.next_value()?,
                            schema,
                        )),
                    ) {
                        return Err(serde::de::Error::custom(err));
                    }
                }
                TypeVariant::Struct(_) => {
                    let builder = struct_builder
                        .reborrow()
                        .init(field)
                        .map_err(serde::de::Error::custom)?;
                    let seed = StructSeed {
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
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("CapnpSerdeVisitor::visit_u16 {v:?}");
        // if let Err(err) = self
        //     .builder
        //     .set(self.field, dynamic_value::Reader::UInt16(v))
        // {
        //     return Err(serde::de::Error::custom(err));
        // }
        Ok(())
    }
}

// Sequences only know their length at deserialization time, so we have to delay
// the initialization of the field
struct CapnpSerdeSeqVisitor<F> {
    inner_ty: capnp::introspect::Type,
    generator: F,
}

impl<'a, 'de, F> Visitor<'de> for CapnpSerdeSeqVisitor<F>
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
                Err(err) => return Err(serde::de::Error::custom(err)),
            };
            let mut index = 0;
            let mut seed = CapnpSerdeElementSeed {
                list_builder,
                index: 0,
                ty: self.inner_ty,
            };
            loop {
                seed.index = index;
                match seq.next_element_seed(&mut seed) {
                    Err(err) => return Err(err),
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

impl<'a, 'de, F> DeserializeSeed<'de> for CapnpSerdeSeqVisitor<F>
where
    F: FnOnce(u32) -> capnp::Result<capnp::dynamic_list::Builder<'a>>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)?;
        Ok(())
    }
}

struct CapnpSerdeTextVisitor<F> {
    setter: F,
}

impl<'de, F, Value> Visitor<'de> for CapnpSerdeTextVisitor<F>
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
        trace!("CapnpSerdeTextVisitor::visit_str {v:?}");
        Ok((self.setter)(v))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("CapnpSerdeTextVisitor::visit_string {v:?}");
        Ok((self.setter)(&v))
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("CapnpSerdeTextVisitor::visit_borrowed_str {v:?}");
        Ok((self.setter)(v))
    }
}

struct CapnpSerdeBytesVisitor<F> {
    setter: F,
}

impl<'de, F, Value> Visitor<'de> for CapnpSerdeBytesVisitor<F>
where
    F: FnOnce(&[u8]) -> Value,
{
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "text")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("CapnpSerdeTextVisitor::visit_bytes {v:?}");
        Ok((self.setter)(v))
    }
}

struct CapnpSerdeElementSeed<'a> {
    list_builder: capnp::dynamic_list::Builder<'a>,
    index: u32,
    ty: capnp::introspect::Type,
}

impl<'a, 'de> DeserializeSeed<'de> for &mut CapnpSerdeElementSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        trace!("CapnpSerdeElementSeed::deserialize {:?}", self.ty);
        match self.ty.which() {
            TypeVariant::List(inner_ty) => {
                let list_builder = self.list_builder.reborrow();
                let seed = CapnpSerdeSeqVisitor {
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
                seed.deserialize(deserializer)?;
            }
            TypeVariant::Text => {
                let list_builder = self.list_builder.reborrow();
                deserializer
                    .deserialize_str(CapnpSerdeTextVisitor {
                        setter: |s: &str| -> capnp::Result<()> {
                            let dynamic_value::Builder::Text(mut text_builder) =
                                list_builder.init(self.index, s.len() as _)?
                            else {
                                return Err(capnp::Error::failed("Internal error".to_owned()));
                            };

                            text_builder.push_str(s);
                            Ok(())
                        },
                    })?
                    .map_err(serde::de::Error::custom)?;
            }
            TypeVariant::Data => {
                deserializer
                    .deserialize_bytes(CapnpSerdeBytesVisitor {
                        setter: |s: &[u8]| -> capnp::Result<()> {
                            self.list_builder
                                .set(self.index, dynamic_value::Reader::Data(s))?;
                            Ok(())
                        },
                    })?
                    .map_err(serde::de::Error::custom)?;
            }
            _ => {
                trace!("CapnpSerdeElementSeed::deserialize other");
                let seed = StructSeed {
                    builder: self
                        .list_builder
                        .reborrow()
                        .get(self.index)
                        .map_err(serde::de::Error::custom)?,
                    ty: self.ty,
                };
                seed.deserialize(deserializer)?;
            }
        }

        Ok(())
    }
}

fn dynamic_value_type_to_str(value: &capnp::dynamic_value::Builder<'_>) -> &'static str {
    match value {
        dynamic_value::Builder::Void => "void",
        dynamic_value::Builder::Bool(_) => "bool",
        dynamic_value::Builder::Int8(_) => "int8",
        dynamic_value::Builder::Int16(_) => "int16",
        dynamic_value::Builder::Int32(_) => "int32",
        dynamic_value::Builder::Int64(_) => "int64",
        dynamic_value::Builder::UInt8(_) => "uint8",
        dynamic_value::Builder::UInt16(_) => "uint16",
        dynamic_value::Builder::UInt32(_) => "uint32",
        dynamic_value::Builder::UInt64(_) => "uint64",
        dynamic_value::Builder::Float32(_) => "float32",
        dynamic_value::Builder::Float64(_) => "float64",
        dynamic_value::Builder::Enum(_) => "enum",
        dynamic_value::Builder::Text(_) => "text",
        dynamic_value::Builder::Data(_) => "data",
        dynamic_value::Builder::Struct(_) => "struct",
        dynamic_value::Builder::List(_) => "list",
        dynamic_value::Builder::AnyPointer(_) => "any",
        dynamic_value::Builder::Capability(_) => "capability",
    }
}

fn type_variant_to_str(var: TypeVariant) -> &'static str {
    match var {
        TypeVariant::Void => "void",
        TypeVariant::Bool => "bool",
        TypeVariant::Int8 => "int8",
        TypeVariant::Int16 => "int16",
        TypeVariant::Int32 => "int32",
        TypeVariant::Int64 => "int64",
        TypeVariant::UInt8 => "uint8",
        TypeVariant::UInt16 => "uint16",
        TypeVariant::UInt32 => "uint32",
        TypeVariant::UInt64 => "uint64",
        TypeVariant::Float32 => "float32",
        TypeVariant::Float64 => "float64",
        TypeVariant::Text => "text",
        TypeVariant::Data => "data",
        TypeVariant::Struct(_) => "struct",
        TypeVariant::AnyPointer => "any",
        TypeVariant::Capability => "capability",
        TypeVariant::Enum(_) => "enum",
        TypeVariant::List(_) => "list",
    }
}
