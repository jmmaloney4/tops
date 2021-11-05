use libipld::Ipld;
use serde::{de, ser, Serialize};
// use anyhow::{Error};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;

pub struct IpldSerializer {
    output: Ipld,
}

pub type Result<T> = std::result::Result<T, Error>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(format!("{}", self))
    }
}

impl<'a> ser::Serializer for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Ipld> {
        self.output = Ipld::Bool(v);
        Ok(self.output)
    }

    fn serialize_i8(self, v: i8) -> Result<Ipld> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Ipld> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Ipld> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Ipld> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Ipld> {
        self.output = Ipld::Integer(v);
        Ok(self.output)
    }

    fn serialize_u8(self, v: u8) -> Result<Ipld> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Ipld> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Ipld> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Ipld> {
        self.output = Ipld::Integer(v.into());
        Ok(self.output)
    }

    fn serialize_f32(self, v: f32) -> Result<Ipld> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Ipld> {
        self.output = Ipld::Float(v);
        Ok(self.output)
    }

    fn serialize_char(self, v: char) -> Result<Ipld> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Ipld> {
        self.output = Ipld::String(String::from(v));
        Ok(self.output)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Ipld> {
        self.output = Ipld::Bytes(Vec::<u8>::from(v));
        Ok(self.output)
    }

    fn serialize_none(self) -> Result<Ipld> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Ipld>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Ipld> {
        self.output = Ipld::Null;
        Ok(self.output)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Ipld> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Ipld> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Ipld>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Ipld>
    where
        T: ?Sized + Serialize,
    {
        self.output = Ipld::StringMap(BTreeMap::from([(
            String::from(_name),
            value.serialize(&mut *self)?,
        )]));
        Ok(self.output)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output = Ipld::List(Vec::<Ipld>::new());
        Ok(self)
    }

    fn serialize_tuple(self, _len: Option<usize>) -> Result<Self::SerializeTuple> {
        self.output = Ipld::List(Vec::<Ipld>::new());
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut IpldSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<Ipld>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::List(v) => {
                let o = value.serialize(*self);
                v.push(o?);
                o
            }
            _ => {
                panic!("Expected List!");
            }
        }
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut IpldSerializer {}

impl<'a> ser::SerializeTupleStruct for &'a mut IpldSerializer {}
