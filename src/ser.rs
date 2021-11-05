use super::error::{Error, Result};
use libipld::Ipld;
use serde::{ser, Serialize};
use std::collections::BTreeMap;

pub struct IpldSerializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: Ipld,
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.

// pub fn to_string<T>(value: &T) -> Result<String>
// where
//     T: Serialize,
// {
//     let mut serializer = Serializer {
//         output: String::new(),
//     };
//     value.serialize(&mut serializer)?;
//     Ok(serializer.output)
// }
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

    // ()
    fn serialize_unit(self) -> Result<Ipld> {
        self.output = Ipld::Null;
        Ok(self.output)
    }

    // Struct w/o data
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Ipld> {
        self.serialize_unit()
    }

    // Enum value w/o associated data
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
        self.output = Ipld::List(Vec::<Ipld>::with_capacity(match _len {
            Some(_len) => _len,
            None => 0,
        }));
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.output = Ipld::List(Vec::<Ipld>::with_capacity(_len));
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTuple> {
        self.serialize_tuple(_len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output = Ipld::StringMap(BTreeMap::from([(
            String::from(_name),
            Ipld::List(Vec::<Ipld>::with_capacity(_len)),
        )]));
        Ok(self)
    }

    // [[K, V], [K, V], ...]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output = Ipld::List(Vec::<Ipld>::with_capacity(match _len {
            Some(_len) => _len,
            None => 0,
        }));
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.output = Ipld::StringMap(BTreeMap::new());
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ipld::StringMap(BTreeMap::from([(
            String::from(_name),
            Ipld::StringMap(BTreeMap::new()),
        )]));
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::List(v) => {
                let o = value.serialize(*self);
                v.push(o?);
                Ok(())
            }
            _ => {
                panic!("Expected List!");
            }
        }
    }

    fn end(self) -> Result<Ipld> {
        Ok(self.output)
    }
}

impl<'a> ser::SerializeTuple for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Ipld> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Ipld> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::StringMap(m) => {
                match m.iter().next() {
                    Some(entry) => match entry.1 {
                        Ipld::List(v) => {
                            v.push(value.serialize(*self)?);
                        }
                        _ => {
                            panic!("Expected List");
                        }
                    },
                    _ => {
                        panic!("Expected Non-Empty Map")
                    }
                }
                Ok(())
            }
            _ => {
                panic!("Expected Map");
            }
        }
    }

    fn end(self) -> Result<Ipld> {
        Ok(self.output)
    }
}

impl<'a> ser::SerializeMap for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::List(v) => {
                let pair = Vec::<Ipld>::with_capacity(2);
                pair.push(key.serialize(*self)?);
                v.push(Ipld::List(pair));
                assert_eq!(self.output, Ipld::List(v));
                Ok(())
            }
            _ => {
                panic!("Expected List");
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::List(v) => match v[v.len() - 1] {
                Ipld::List(pair) => {
                    pair.push(value.serialize(*self)?);
                    Ok(())
                }
                _ => {
                    panic!("Expected List")
                }
            },
            _ => {
                panic!("Expected List");
            }
        }
    }

    fn end(self) -> Result<Ipld> {
        Ok(self.output)
    }
}

impl<'a> ser::SerializeStruct for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        
        match self.output {
            Ipld::StringMap(m) => {
                m.insert(String::from(key), value.serialize(*self)?);
            }
            _ => {
                panic!("Expected Map");
            }
        }

        Ok(())
    }

    fn end(self) -> Result<Ipld> {
        Ok(self.output)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut IpldSerializer {
    type Ok = Ipld;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.output {
            Ipld::StringMap(top) => {
                match top.iter().next() {
                    Some(entry) => {
                        match entry.1 {
                            Ipld::StringMap(m) => {
                                m.insert(String::from(key), value.serialize(*self)?);
                            }
                            _ => {
                                panic!("Expected Map")
                            }
                        }
                    }
                    _ => {
                        panic!("Expected Non-Empty Map");
                    }
                }
            }
            _ => {
                panic!("Expected Map")
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Ipld> {
        Ok(self.output)
    }
}
