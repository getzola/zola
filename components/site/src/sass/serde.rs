use config::Config;
use errors::{Context, Result};

use serde::{Serialize, Serializer};

use std::fmt::Display;

/// write the site's `Config` struct out as a SCSS map literal.  this
/// first converts the `Config` to a `SassConfig` to only retain and
/// serialize the keys in the config document that we want to end up
/// in the resulting SCSS map.
pub(crate) fn serialize_config(config: &Config) -> Result<String> {
    let sass_config = config.sass_config();
    let mut ser = SassMapSerializer::default();

    serde::Serialize::serialize(&sass_config, &mut ser)
        .context("failed to serialize Zola config document")?;

    Ok(ser.output)
}

/// custom serde `Serializer` that serializes a structure as an SCSS
/// map literal.  the primary difference between an SCSS map literal
/// and JSON is that SCSS uses parentheses like `("key": "value")`
/// instead of `{"key": "value"}` to express maps/dictionaries.
#[derive(Default)]
struct SassMapSerializer {
    output: String,
}

#[derive(Debug)]
struct SassMapSerializerError(String);

type SassMapSerializerResult = std::result::Result<(), SassMapSerializerError>;

impl serde::ser::Error for SassMapSerializerError {
    fn custom<T: Display>(msg: T) -> Self {
        SassMapSerializerError(msg.to_string())
    }
}

impl Display for SassMapSerializerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("SassMapSerializerError({})", self.0))
    }
}

impl std::error::Error for SassMapSerializerError {}

impl<'a> Serializer for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    /// serialize any chars as if they were single-character strings
    fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    /// not implemented, as the type being serialized here is TOML-based, which
    /// has no native byte type
    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    /// treat None as null
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }

    /// treat Some(T) just as an instance of T itself
    fn serialize_some<T: ?Sized>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    /// treat the unit struct `()` as None/null
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    /// treat the unit struct `()` as None/null
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    /// for a unit variant like `MyEnum::A`, just serialize the variant name
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    /// for a newtype struct like `Dollars(u8)`, just serialize as
    /// the wrapped type
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    /// for a newtype variant like `Currency::Dollars(u8)`, just serialize as
    /// the wrapped type
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    /// arrays/sequences are serialized as in JSON
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        self.output += "[";
        Ok(self)
    }

    /// treat a tuple the same way we treat an array/sequence
    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    /// treat a tuple struct like `Rgb(u8, u8, u8)` the same way we treat
    /// an array/sequence
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    /// treat a tuple variant like `Color::Rgb(u8, u8, u8)` the same way
    /// we treat an array/sequence
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    /// serialize maps as `("key": "value")`
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        self.output += "(";
        Ok(self)
    }

    /// treat a struct with named members as a map
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    /// treat a struct variant like `Color::Rgb { r: u8, g: u8, b: u8}`
    /// as a map
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut SassMapSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = SassMapSerializerError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        // output a comma for all but the first element
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> SassMapSerializerResult {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> serde::ser::SerializeTuple for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        // output a comma for all but the first element
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> serde::ser::SerializeTupleStruct for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        // output a comma for all but the first element
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple variants.
impl<'a> serde::ser::SerializeTupleVariant for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        // output a comma for all but the first element
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += "]";
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_key<T>(&mut self, key: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('(') {
            self.output += ",";
        }
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += ")";
        Ok(())
    }
}

// Same thing but for structs, where the keys are static string field names.
impl<'a> serde::ser::SerializeStruct for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('(') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += ")";
        Ok(())
    }
}

// Same thing but for struct variants, where we ignore the variant name.
impl<'a> serde::ser::SerializeStructVariant for &'a mut SassMapSerializer {
    type Ok = ();
    type Error = SassMapSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> SassMapSerializerResult
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('(') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> SassMapSerializerResult {
        self.output += ")";
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        verify_serialization(true, "true");
        verify_serialization(64, "64");
        verify_serialization(-123, "-123");
        verify_serialization(567.89, "567.89");
        verify_serialization('t', "\"t\"");
        verify_serialization("abc", "\"abc\"");
        verify_serialization(Option::<i32>::None, "null");
        verify_serialization((), "null");
    }

    #[test]
    fn test_arrays() {
        verify_serialization(&[123, 456, 789], "[123,456,789]");
        verify_serialization((123, 456, 789), "[123,456,789]");
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Inner {
            a: Vec<i32>,
            b: (String, f64),
        }

        #[derive(Serialize)]
        struct Outer {
            a: i32,
            b: String,
            c: Inner,
        }

        let val = Outer {
            a: 42,
            b: "abc".to_string(),
            c: Inner { a: vec![6, 7, 8], b: ("def".to_string(), 123.45) },
        };

        let expected = "(\"a\":42,\"b\":\"abc\",\"c\":(\"a\":[6,7,8],\"b\":[\"def\",123.45]))";
        verify_serialization(val, expected);
    }

    fn verify_serialization<T: Serialize>(val: T, expected: &str) {
        let mut ser = SassMapSerializer::default();
        val.serialize(&mut ser).unwrap();
        assert_eq!(ser.output, expected);
    }
}
