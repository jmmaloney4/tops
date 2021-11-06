use std::any::type_name;
use std::ops::{AddAssign, MulAssign, Neg};

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use crate::error::{Error, Result};

use libipld::Ipld;

pub struct IpldDeserializer<'de> {
    input: &'de Ipld,
}

impl<'de> IpldDeserializer<'de> {
    pub fn from_ipld(input: &'de Ipld) -> Self {
        IpldDeserializer { input }
    }
}

pub fn from_ipld<'a, T>(input: &'a Ipld) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = IpldDeserializer::from_ipld(input);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
impl<'de> IpldDeserializer<'de> {
    // // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(Error::Eof)
    // }

    // // Consume the first character in the input.
    // fn next_char(&mut self) -> Result<char> {
    //     let ch = self.peek_char()?;
    //     self.input = &self.input[ch.len_utf8()..];
    //     Ok(ch)
    // }

    // // Parse the JSON identifier `true` or `false`.
    // fn parse_bool(&mut self) -> Result<bool> {
    //     if self.input.starts_with("true") {
    //         self.input = &self.input["true".len()..];
    //         Ok(true)
    //     } else if self.input.starts_with("false") {
    //         self.input = &self.input["false".len()..];
    //         Ok(false)
    //     } else {
    //         Err(Error::ExpectedBoolean)
    //     }
    // }

    // // Parse a group of decimal digits as an unsigned integer of type T.
    // //
    // // This implementation is a bit too lenient, for example `001` is not
    // // allowed in JSON. Also the various arithmetic operations can overflow and
    // // panic or return bogus data. But it is good enough for example code!
    // fn parse_unsigned<T>(&mut self) -> Result<T>
    // where
    //     T: AddAssign<T> + MulAssign<T> + From<u8>,
    // {
    //     let mut int = match self.next_char()? {
    //         ch @ '0'..='9' => T::from(ch as u8 - b'0'),
    //         _ => {
    //             return Err(Error::ExpectedInteger);
    //         }
    //     };
    //     loop {
    //         match self.input.chars().next() {
    //             Some(ch @ '0'..='9') => {
    //                 self.input = &self.input[1..];
    //                 int *= T::from(10);
    //                 int += T::from(ch as u8 - b'0');
    //             }
    //             _ => {
    //                 return Ok(int);
    //             }
    //         }
    //     }
    // }

    // // Parse a possible minus sign followed by a group of decimal digits as a
    // // signed integer of type T.
    // fn parse_signed<T>(&mut self) -> Result<T>
    // where
    //     T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    // {
    //     // Optional minus sign, delegate to `parse_unsigned`, negate if negative.
    //     unimplemented!()
    // }

    // // Parse a string until the next '"' character.
    // //
    // // Makes no attempt to handle escape sequences. What did you expect? This is
    // // example code!
    // fn parse_string(&mut self) -> Result<&'de str> {
    //     if self.next_char()? != '"' {
    //         return Err(Error::ExpectedString);
    //     }
    //     match self.input.find('"') {
    //         Some(len) => {
    //             let s = &self.input[..len];
    //             self.input = &self.input[len + 1..];
    //             Ok(s)
    //         }
    //         None => Err(Error::Eof),
    //     }
    // }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + TryFrom<i128>,
    {
        match self.input {
            Ipld::Integer(i) => match TryInto::<T>::try_into(*i) {
                Err(e) => {
                    return Err(Error::Message(format!(
                        "Couldn't covert {} into {:?}",
                        i,
                        type_name::<T>()
                    )));
                }
                Ok(i) => Ok(i),
            },
            _ => Err(Error::ExpectedInteger),
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + TryFrom<u64>,
    {
        match self.input {
            Ipld::Integer(i) => match TryInto::<T>::try_into(*i as u64) {
                Err(e) => {
                    return Err(Error::Message(format!(
                        "Couldn't covert {} into {:?}",
                        i,
                        type_name::<T>()
                    )));
                }
                Ok(i) => Ok(i),
            },
            _ => Err(Error::ExpectedInteger),
        }
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + TryFrom<f64>,
    {
        match self.input {
            Ipld::Float(f) => match TryInto::<T>::try_into(*f) {
                Err(e) => {
                    return Err(Error::Message(format!(
                        "Couldn't covert {} into {:?}",
                        f,
                        type_name::<T>()
                    )));
                }
                Ok(f) => Ok(f),
            },
            _ => Err(Error::ExpectedFloat),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        match self.input {
            Ipld::String(s) => Ok(*s),
            _ => Err(Error::ExpectedString),
        }
    }

    fn parse_bytes(&mut self) -> Result<Vec<u8>> {
        match self.input {
            Ipld::Bytes(b) => Ok(*b),
            _ => Err(Error::ExpectedBytes),
        }
    }

    fn parse_null(&mut self) -> Result<()> {
        match self.input {
            Ipld::Null => Ok(()),
            _ => Err(Error::ExpectedNull),
        }
    }

    fn parse_option(&mut self) -> Result<Option<Ipld>> {
        match self.input {
            Ipld::Null => Ok(None),
            Ipld => Ok(Some(*self.input)),
            _ => Err(Error::ExpectedOption),
        }
    }

    fn parse_list(&mut self) -> Result<Vec<Ipld>> {
        match self.input {
            Ipld::List(v) => Ok(*v),
            _ => Err(Error::ExpectedList),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut IpldDeserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Ipld::Bool(b) => visitor.visit_bool(*b),
            _ => {
                panic!("Expected Bool");
            }
        }
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        let s = self.parse_string()?;
        let chars = s.chars().collect::<Vec<char>>();
        if chars.len() != 1 {
            Err(Error::ExpectedChar)
        } else {
            visitor.visit_char(chars[0])
        }
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(&self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(&self.parse_bytes()?)
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_option() {
            Ok(None) => {
                visitor.visit_none()
            },
            Ok(Some(_)) => {
                visitor.visit_some(self)
            }
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_null() {
            Ok(_) => {
                visitor.visit_unit()
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_list()?

        // Parse the opening bracket of the sequence.
        if self.next_char()? == '[' {
            // Give the visitor access to each element of the sequence.
            let value = visitor.visit_seq(CommaSeparated::new(self))?;
            // Parse the closing bracket of the sequence.
            if self.next_char()? == ']' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse the opening brace of the map.
        if self.next_char()? == '{' {
            // Give the visitor access to each entry of the map.
            let value = visitor.visit_map(CommaSeparated::new(self))?;
            // Parse the closing brace of the map.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_char()? == '"' {
            // Visit a unit variant.
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if self.next_char()? == '{' {
            // Visit a newtype variant, tuple variant, or struct variant.
            let value = visitor.visit_enum(Enum::new(self))?;
            // Parse the matching close brace.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedEnum)
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}



// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { de, first: true }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de> SeqAccess<'de> for IpldDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {

        match self.parse_list()? {
            Ipld::List(v) => {
                
            }
        }

        // Check if there are no more elements.
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedArrayComma);
        }
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // Comma is required before every entry except the first.
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedMapComma);
        }
        self.first = false;
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        if self.de.next_char()? != ':' {
            return Err(Error::ExpectedMapColon);
        }
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        let val = seed.deserialize(&mut *self.de)?;
        // Parse the colon separating map key from value.
        if self.de.next_char()? == ':' {
            Ok((val, self))
        } else {
            Err(Error::ExpectedMapColon)
        }
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
