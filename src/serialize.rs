use capnp::dynamic_value;
use serde::ser::{Error as SerdeError, SerializeMap, SerializeSeq};
use tracing::trace;

/// A type that can be used to serialize a Cap'n Proto dynamic value into any serde-implementing format.
///
/// This can be used to convert a Cap'n Proto message to any format that implements serde, such as JSON, YAML or CBOR.
///
/// # Example
///
///
/// ```rust
/// use capnp_serde::CapnpSerdeReader;
///
/// let value = CapnpSerdeReader::from(capnp::dynamic_value::Reader::from(42));
/// let json = serde_json::to_string(&value).unwrap();
/// assert_eq!(json, "42");
/// ```
#[repr(transparent)]
pub struct CapnpSerdeReader<'a>(dynamic_value::Reader<'a>);

impl<'a, R> From<R> for CapnpSerdeReader<'a>
where
    dynamic_value::Reader<'a>: From<R>,
{
    /// Creates a `CapnpSerdeReader` from a `capnp::dynamic_value::Reader`.
    ///
    /// This is the initializer for `CapnpSerdeReader`.
    fn from(reader: R) -> Self {
        Self(reader.into())
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
                // This filter excludes fields with their default value set and non-active union fields
                let fields: Box<[_]> = fields
                    .into_iter()
                    .filter(|&field| reader.has(field).unwrap_or_default())
                    .collect();
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
