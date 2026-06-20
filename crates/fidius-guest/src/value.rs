// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `Value` — a neutral, self-describing value tree for the plugin boundary.
//!
//! Every fidius execution backend dispatches typed method calls through a
//! single `PluginExecutor::call(method, Value) -> Value` seam (see
//! `fidius-host`). `Value` is the lingua franca each backend maps to its
//! native representation:
//!
//! - **cdylib**: `Value` → bincode → vtable FFI → bincode → `Value`
//! - **python**: `Value` → `PyObject` → call → `PyObject` → `Value`
//! - **wasm**: `Value` → `wasmtime` `component::Val` (Canonical ABI) → `Value`
//!
//! The host's generic `call_method<I, O>` stays caller-identical by going
//! `I → Value` ([`to_value`]) then `Value → O` ([`from_value`]) around the
//! executor.
//!
//! **Layering rule:** `Value` lives in `fidius-core` and is deliberately
//! free of any backend dependency. Only the WASM executor (Phase 2) maps it
//! to `wasmtime::component::Val`; cdylib and Python never see wasmtime.
//!
//! The variant set mirrors the WebAssembly Component Model value space so the
//! Phase-2 mapping is mechanical: distinct signed/unsigned integer widths,
//! `f32`/`f64`, `char`, `string`, `list<u8>` as [`Value::Bytes`], lists,
//! records, options, and variants.

use std::fmt;

use serde::{de, ser, Deserialize, Serialize};

/// A self-describing value crossing the plugin-call boundary.
///
/// Construct one from any `Serialize` type with [`to_value`] and read it back
/// into any `DeserializeOwned` type with [`from_value`].
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Boolean.
    Bool(bool),
    /// Signed 8-bit integer.
    S8(i8),
    /// Signed 16-bit integer.
    S16(i16),
    /// Signed 32-bit integer.
    S32(i32),
    /// Signed 64-bit integer.
    S64(i64),
    /// Unsigned 8-bit integer.
    U8(u8),
    /// Unsigned 16-bit integer.
    U16(u16),
    /// Unsigned 32-bit integer.
    U32(u32),
    /// Unsigned 64-bit integer.
    U64(u64),
    /// 32-bit float.
    F32(f32),
    /// 64-bit float.
    F64(f64),
    /// Unicode scalar value.
    Char(char),
    /// UTF-8 string.
    String(String),
    /// Opaque byte string (`list<u8>` in WIT terms).
    Bytes(Vec<u8>),
    /// Optional value (`none`/`some`).
    Option(Option<Box<Value>>),
    /// Ordered sequence — serde seqs, tuples, and tuple structs land here.
    List(Vec<Value>),
    /// Named fields — serde structs and string-keyed maps land here.
    /// Field order is preserved (insertion order).
    Record(Vec<(String, Value)>),
    /// General key/value map for non-string keys.
    Map(Vec<(Value, Value)>),
    /// A tagged enum case. `value` carries the payload: [`Value::Unit`] for a
    /// unit variant, the inner value for a newtype variant, a [`Value::List`]
    /// for a tuple variant, or a [`Value::Record`] for a struct variant.
    Variant {
        /// The variant's name.
        name: String,
        /// The variant's payload.
        value: Box<Value>,
    },
    /// The unit value — serde `()` and unit structs.
    Unit,
}

/// Error produced while converting to or from [`Value`].
#[derive(Debug, thiserror::Error)]
#[error("value conversion error: {0}")]
pub struct ValueError(pub String);

impl ser::Error for ValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ValueError(msg.to_string())
    }
}

impl de::Error for ValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ValueError(msg.to_string())
    }
}

/// Convert any [`Serialize`] type into a [`Value`].
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, ValueError> {
    value.serialize(ValueSerializer)
}

/// Convert a [`Value`] into any [`Deserialize`] type.
pub fn from_value<T>(value: Value) -> Result<T, ValueError>
where
    T: de::DeserializeOwned,
{
    T::deserialize(value)
}

// ===========================================================================
// Serializer: T -> Value
// ===========================================================================

struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = ValueError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value, ValueError> {
        Ok(Value::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Value, ValueError> {
        Ok(Value::S8(v))
    }
    fn serialize_i16(self, v: i16) -> Result<Value, ValueError> {
        Ok(Value::S16(v))
    }
    fn serialize_i32(self, v: i32) -> Result<Value, ValueError> {
        Ok(Value::S32(v))
    }
    fn serialize_i64(self, v: i64) -> Result<Value, ValueError> {
        Ok(Value::S64(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Value, ValueError> {
        Ok(Value::U8(v))
    }
    fn serialize_u16(self, v: u16) -> Result<Value, ValueError> {
        Ok(Value::U16(v))
    }
    fn serialize_u32(self, v: u32) -> Result<Value, ValueError> {
        Ok(Value::U32(v))
    }
    fn serialize_u64(self, v: u64) -> Result<Value, ValueError> {
        Ok(Value::U64(v))
    }
    fn serialize_f32(self, v: f32) -> Result<Value, ValueError> {
        Ok(Value::F32(v))
    }
    fn serialize_f64(self, v: f64) -> Result<Value, ValueError> {
        Ok(Value::F64(v))
    }
    fn serialize_char(self, v: char) -> Result<Value, ValueError> {
        Ok(Value::Char(v))
    }
    fn serialize_str(self, v: &str) -> Result<Value, ValueError> {
        Ok(Value::String(v.to_string()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value, ValueError> {
        Ok(Value::Bytes(v.to_vec()))
    }
    fn serialize_none(self) -> Result<Value, ValueError> {
        Ok(Value::Option(None))
    }
    fn serialize_some<T>(self, value: &T) -> Result<Value, ValueError>
    where
        T: ?Sized + Serialize,
    {
        Ok(Value::Option(Some(Box::new(
            value.serialize(ValueSerializer)?,
        ))))
    }
    fn serialize_unit(self) -> Result<Value, ValueError> {
        Ok(Value::Unit)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, ValueError> {
        Ok(Value::Unit)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, ValueError> {
        Ok(Value::Variant {
            name: variant.to_string(),
            value: Box::new(Value::Unit),
        })
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, ValueError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(ValueSerializer)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, ValueError>
    where
        T: ?Sized + Serialize,
    {
        Ok(Value::Variant {
            name: variant.to_string(),
            value: Box::new(value.serialize(ValueSerializer)?),
        })
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, ValueError> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer, ValueError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, ValueError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantSerializer, ValueError> {
        Ok(TupleVariantSerializer {
            name: variant.to_string(),
            items: Vec::with_capacity(len),
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer, ValueError> {
        Ok(MapSerializer {
            entries: Vec::new(),
            next_key: None,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<StructSerializer, ValueError> {
        Ok(StructSerializer {
            fields: Vec::with_capacity(len),
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<StructVariantSerializer, ValueError> {
        Ok(StructVariantSerializer {
            name: variant.to_string(),
            fields: Vec::with_capacity(len),
        })
    }
}

struct SeqSerializer {
    items: Vec<Value>,
}
impl ser::SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, ValueError> {
        Ok(Value::List(self.items))
    }
}
impl ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Value, ValueError> {
        ser::SerializeSeq::end(self)
    }
}
impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Value, ValueError> {
        ser::SerializeSeq::end(self)
    }
}

struct TupleVariantSerializer {
    name: String,
    items: Vec<Value>,
}
impl ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, ValueError> {
        Ok(Value::Variant {
            name: self.name,
            value: Box::new(Value::List(self.items)),
        })
    }
}

struct MapSerializer {
    entries: Vec<(Value, Value)>,
    next_key: Option<Value>,
}
impl ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(ValueSerializer)?);
        Ok(())
    }
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| ValueError("serialize_value called before serialize_key".into()))?;
        self.entries.push((key, value.serialize(ValueSerializer)?));
        Ok(())
    }
    fn end(self) -> Result<Value, ValueError> {
        // If every key is a string, prefer a Record so round-tripping into
        // structs (and the Python/JSON bridges) is natural.
        if self
            .entries
            .iter()
            .all(|(k, _)| matches!(k, Value::String(_)))
        {
            let fields = self
                .entries
                .into_iter()
                .map(|(k, v)| match k {
                    Value::String(s) => (s, v),
                    _ => unreachable!(),
                })
                .collect();
            Ok(Value::Record(fields))
        } else {
            Ok(Value::Map(self.entries))
        }
    }
}

struct StructSerializer {
    fields: Vec<(String, Value)>,
}
impl ser::SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.fields
            .push((key.to_string(), value.serialize(ValueSerializer)?));
        Ok(())
    }
    fn end(self) -> Result<Value, ValueError> {
        Ok(Value::Record(self.fields))
    }
}

struct StructVariantSerializer {
    name: String,
    fields: Vec<(String, Value)>,
}
impl ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = ValueError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), ValueError>
    where
        T: ?Sized + Serialize,
    {
        self.fields
            .push((key.to_string(), value.serialize(ValueSerializer)?));
        Ok(())
    }
    fn end(self) -> Result<Value, ValueError> {
        Ok(Value::Variant {
            name: self.name,
            value: Box::new(Value::Record(self.fields)),
        })
    }
}

// ===========================================================================
// Deserializer: Value -> T
// ===========================================================================

impl<'de> de::Deserializer<'de> for Value {
    type Error = ValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Bool(b) => visitor.visit_bool(b),
            Value::S8(v) => visitor.visit_i8(v),
            Value::S16(v) => visitor.visit_i16(v),
            Value::S32(v) => visitor.visit_i32(v),
            Value::S64(v) => visitor.visit_i64(v),
            Value::U8(v) => visitor.visit_u8(v),
            Value::U16(v) => visitor.visit_u16(v),
            Value::U32(v) => visitor.visit_u32(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::F32(v) => visitor.visit_f32(v),
            Value::F64(v) => visitor.visit_f64(v),
            Value::Char(v) => visitor.visit_char(v),
            Value::String(s) => visitor.visit_string(s),
            Value::Bytes(b) => visitor.visit_byte_buf(b),
            Value::Unit => visitor.visit_unit(),
            Value::Option(None) => visitor.visit_none(),
            Value::Option(Some(v)) => visitor.visit_some(*v),
            Value::List(items) => visitor.visit_seq(SeqAccess {
                iter: items.into_iter(),
            }),
            Value::Record(fields) => visitor.visit_map(RecordAccess {
                iter: fields.into_iter(),
                value: None,
            }),
            Value::Map(entries) => visitor.visit_map(MapAccess {
                iter: entries.into_iter(),
                value: None,
            }),
            Value::Variant { name, value } => visitor.visit_map(SingletonMapAccess {
                key: Some(name),
                value: Some(*value),
            }),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Option(None) => visitor.visit_none(),
            Value::Option(Some(v)) => visitor.visit_some(*v),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self {
            // `EnumName::Variant` with no payload may serialize as a bare
            // string in some formats; accept that too.
            Value::String(s) => visitor.visit_enum(EnumAccess {
                name: s,
                value: Value::Unit,
            }),
            Value::Variant { name, value } => visitor.visit_enum(EnumAccess {
                name,
                value: *value,
            }),
            other => Err(ValueError(format!(
                "expected enum variant, found {}",
                other.kind()
            ))),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Unit => visitor.visit_unit(),
            // An empty tuple/list also reads as unit.
            Value::List(items) if items.is_empty() => visitor.visit_unit(),
            other => Err(ValueError(format!("expected unit, found {}", other.kind()))),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Map(entries) => visitor.visit_map(MapAccess {
                iter: entries.into_iter(),
                value: None,
            }),
            Value::Record(fields) => visitor.visit_map(RecordAccess {
                iter: fields.into_iter(),
                value: None,
            }),
            // A map projected across the WASM boundary arrives as `list<tuple<k, v>>`
            // — a `Value::List` of 2-element pairs. Accept it as a map so
            // `HashMap`/`BTreeMap` round-trips (PC.1). `Vec<(K, V)>` still reads the
            // same value via `deserialize_seq`.
            Value::List(items) => {
                let mut entries = Vec::with_capacity(items.len());
                for it in items {
                    match it {
                        Value::List(mut kv) if kv.len() == 2 => {
                            let v = kv.pop().unwrap();
                            let k = kv.pop().unwrap();
                            entries.push((k, v));
                        }
                        other => {
                            return Err(ValueError(format!(
                                "expected a [key, value] pair reading a map from a list, found {}",
                                other.kind()
                            )))
                        }
                    }
                }
                visitor.visit_map(MapAccess {
                    iter: entries.into_iter(),
                    value: None,
                })
            }
            other => Err(ValueError(format!("expected map, found {}", other.kind()))),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf seq tuple tuple_struct struct identifier
        ignored_any
    }
}

impl Value {
    fn kind(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::S8(_) | Value::S16(_) | Value::S32(_) | Value::S64(_) => "signed integer",
            Value::U8(_) | Value::U16(_) | Value::U32(_) | Value::U64(_) => "unsigned integer",
            Value::F32(_) | Value::F64(_) => "float",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Option(_) => "option",
            Value::List(_) => "list",
            Value::Record(_) => "record",
            Value::Map(_) => "map",
            Value::Variant { .. } => "variant",
            Value::Unit => "unit",
        }
    }
}

struct SeqAccess {
    iter: std::vec::IntoIter<Value>,
}
impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = ValueError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, ValueError>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(v).map(Some),
            None => Ok(None),
        }
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct RecordAccess {
    iter: std::vec::IntoIter<(String, Value)>,
    value: Option<Value>,
}
impl<'de> de::MapAccess<'de> for RecordAccess {
    type Error = ValueError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                seed.deserialize(Value::String(k)).map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, ValueError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let v = self
            .value
            .take()
            .ok_or_else(|| ValueError("next_value called before next_key".into()))?;
        seed.deserialize(v)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct MapAccess {
    iter: std::vec::IntoIter<(Value, Value)>,
    value: Option<Value>,
}
impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = ValueError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                seed.deserialize(k).map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, ValueError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let v = self
            .value
            .take()
            .ok_or_else(|| ValueError("next_value called before next_key".into()))?;
        seed.deserialize(v)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

/// Presents a `Value::Variant` as a single-entry map for `deserialize_any`
/// consumers (e.g. deserializing into `serde_json::Value`).
struct SingletonMapAccess {
    key: Option<String>,
    value: Option<Value>,
}
impl<'de> de::MapAccess<'de> for SingletonMapAccess {
    type Error = ValueError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.key.take() {
            Some(k) => seed.deserialize(Value::String(k)).map(Some),
            None => Ok(None),
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, ValueError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let v = self
            .value
            .take()
            .ok_or_else(|| ValueError("variant value already consumed".into()))?;
        seed.deserialize(v)
    }
}

struct EnumAccess {
    name: String,
    value: Value,
}
impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = ValueError;
    type Variant = VariantAccess;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantAccess), ValueError>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(Value::String(self.name))?;
        Ok((variant, VariantAccess { value: self.value }))
    }
}

struct VariantAccess {
    value: Value,
}
impl<'de> de::VariantAccess<'de> for VariantAccess {
    type Error = ValueError;
    fn unit_variant(self) -> Result<(), ValueError> {
        match self.value {
            Value::Unit => Ok(()),
            other => Err(ValueError(format!(
                "expected unit variant, found {}",
                other.kind()
            ))),
        }
    }
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, ValueError>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value)
    }
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::List(items) => visitor.visit_seq(SeqAccess {
                iter: items.into_iter(),
            }),
            other => Err(ValueError(format!(
                "expected tuple variant, found {}",
                other.kind()
            ))),
        }
    }
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, ValueError>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Record(fields) => visitor.visit_map(RecordAccess {
                iter: fields.into_iter(),
                value: None,
            }),
            other => Err(ValueError(format!(
                "expected struct variant, found {}",
                other.kind()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn round_trip<T>(value: T)
    where
        T: Serialize + de::DeserializeOwned + PartialEq + fmt::Debug + Clone,
    {
        let v = to_value(&value).expect("to_value");
        let back: T = from_value(v).expect("from_value");
        assert_eq!(back, value);
    }

    #[test]
    fn map_deserializes_from_a_list_of_pairs() {
        use std::collections::HashMap;
        // PC.1: a map crosses the WASM boundary as `list<tuple<k,v>>`, i.e. a
        // `Value::List` of 2-element pairs. It must deserialize into a `HashMap`...
        let pairs = Value::List(vec![
            Value::List(vec![Value::String("a".into()), Value::U32(1)]),
            Value::List(vec![Value::String("b".into()), Value::U32(2)]),
        ]);
        let m: HashMap<String, u32> = from_value(pairs.clone()).expect("list-of-pairs → map");
        assert_eq!(m.get("a"), Some(&1));
        assert_eq!(m.get("b"), Some(&2));
        // ...and the same value still reads as a `Vec<(K, V)>`.
        let v: Vec<(String, u32)> = from_value(pairs).expect("list-of-pairs → vec");
        assert_eq!(v.len(), 2);
        // A non-string-keyed map also round-trips (via Value::Map).
        let mut nk: HashMap<u32, String> = HashMap::new();
        nk.insert(7, "seven".into());
        round_trip(nk);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Greeting {
        name: String,
        times: u32,
        loud: bool,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Wrapper(u64);

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    enum Shape {
        Unit,
        Newtype(i32),
        Tuple(i32, i32),
        Struct { w: u16, h: u16 },
    }

    #[test]
    fn primitives() {
        round_trip(true);
        round_trip(-5i8);
        round_trip(40000u16);
        round_trip(u64::MAX);
        round_trip(i64::MIN);
        round_trip(3.5f32);
        round_trip(2.718281828f64);
        round_trip('λ');
        round_trip("hello".to_string());
    }

    #[test]
    fn collections() {
        round_trip(vec![1i32, 2, 3]);
        round_trip(Some(7u8));
        round_trip(Option::<u8>::None);
        round_trip((1i32, "two".to_string(), false));
        round_trip(vec![Some(1u32), None, Some(3)]);
    }

    #[test]
    fn structs_and_maps() {
        round_trip(Greeting {
            name: "ada".to_string(),
            times: 3,
            loud: true,
        });
        round_trip(Wrapper(99));

        use std::collections::BTreeMap;
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), 1i32);
        m.insert("b".to_string(), 2);
        round_trip(m);

        let mut numkeys = BTreeMap::new();
        numkeys.insert(1u32, "one".to_string());
        numkeys.insert(2u32, "two".to_string());
        round_trip(numkeys);
    }

    #[test]
    fn enums() {
        round_trip(Shape::Unit);
        round_trip(Shape::Newtype(-9));
        round_trip(Shape::Tuple(3, 4));
        round_trip(Shape::Struct { w: 10, h: 20 });
    }

    #[test]
    fn nested() {
        #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
        struct Outer {
            shapes: Vec<Shape>,
            tag: Option<String>,
        }
        round_trip(Outer {
            shapes: vec![Shape::Unit, Shape::Tuple(1, 2), Shape::Newtype(5)],
            tag: Some("x".to_string()),
        });
    }

    #[test]
    fn struct_shape_is_record() {
        let v = to_value(&Greeting {
            name: "z".into(),
            times: 1,
            loud: false,
        })
        .unwrap();
        match v {
            Value::Record(fields) => {
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "times");
                assert_eq!(fields[2].0, "loud");
            }
            other => panic!("expected record, got {other:?}"),
        }
    }
}

// `Value` is itself serde-serializable so it can be embedded in other
// structures and round-tripped through any format when needed.
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::S8(v) => serializer.serialize_i8(*v),
            Value::S16(v) => serializer.serialize_i16(*v),
            Value::S32(v) => serializer.serialize_i32(*v),
            Value::S64(v) => serializer.serialize_i64(*v),
            Value::U8(v) => serializer.serialize_u8(*v),
            Value::U16(v) => serializer.serialize_u16(*v),
            Value::U32(v) => serializer.serialize_u32(*v),
            Value::U64(v) => serializer.serialize_u64(*v),
            Value::F32(v) => serializer.serialize_f32(*v),
            Value::F64(v) => serializer.serialize_f64(*v),
            Value::Char(v) => serializer.serialize_char(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Bytes(v) => serializer.serialize_bytes(v),
            Value::Unit => serializer.serialize_unit(),
            Value::Option(None) => serializer.serialize_none(),
            Value::Option(Some(v)) => serializer.serialize_some(v),
            Value::List(items) => {
                use ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Record(fields) => {
                use ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (k, v) in fields {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Map(entries) => {
                use ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Variant { name, value } => {
                use ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(name, value)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any fidius value")
            }
            fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::S64(v))
            }
            fn visit_i128<E>(self, v: i128) -> Result<Value, E>
            where
                E: de::Error,
            {
                i64::try_from(v)
                    .map(Value::S64)
                    .map_err(|_| E::custom("i128 out of range"))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
                Ok(Value::U64(v))
            }
            fn visit_u128<E>(self, v: u128) -> Result<Value, E>
            where
                E: de::Error,
            {
                u64::try_from(v)
                    .map(Value::U64)
                    .map_err(|_| E::custom("u128 out of range"))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                Ok(Value::F64(v))
            }
            fn visit_char<E>(self, v: char) -> Result<Value, E> {
                Ok(Value::Char(v))
            }
            fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.to_string()))
            }
            fn visit_string<E>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Value, E> {
                Ok(Value::Bytes(v.to_vec()))
            }
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Value, E> {
                Ok(Value::Bytes(v))
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Unit)
            }
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Option(None))
            }
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                Ok(Value::Option(Some(Box::new(Value::deserialize(
                    deserializer,
                )?))))
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut items = Vec::new();
                while let Some(v) = seq.next_element()? {
                    items.push(v);
                }
                Ok(Value::List(items))
            }
            fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut fields = Vec::new();
                while let Some((k, v)) = map.next_entry::<String, Value>()? {
                    fields.push((k, v));
                }
                Ok(Value::Record(fields))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}
