use std::sync::LazyLock;

use capnp::{dynamic_value, introspect::TypeVariant};
use once_map::OnceMap;

pub(crate) mod bools;
pub(crate) mod data;
pub(crate) mod list_element;
pub(crate) mod num;
pub(crate) mod seq;
pub(crate) mod structs;
pub(crate) mod text;
pub(crate) mod void;

static STRUCT_ENUM_SCHEMA_FIELD_NAMES: LazyLock<OnceMap<u64, Box<[&'static str]>>> =
    LazyLock::new(OnceMap::new);

fn dynamic_value_type_to_str(value: &dynamic_value::Builder<'_>) -> &'static str {
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
