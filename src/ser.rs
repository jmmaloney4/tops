use super::error::{Error, Result};
use libipld::Ipld;
use serde::{ser, Serialize};
use std::collections::BTreeMap;

pub struct IpldSerializer {
    output: Ipld,
}

pub fn to_ipld<T>(value: &T) -> Result<Ipld>
where
    T: ?Sized + Serialize,
{
    let mut serializer = IpldSerializer { output: Ipld::Null };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output = Ipld::Bool(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_i128(i128::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.output = Ipld::Integer(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output = Ipld::Integer(v.into());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output = Ipld::Float(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output = Ipld::String(String::from(v));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output = Ipld::Bytes(Vec::<u8>::from(v));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // ()
    fn serialize_unit(self) -> Result<()> {
        self.output = Ipld::Null;
        Ok(())
    }

    // Struct w/o data
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // Enum value w/o associated data
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
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
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output = Ipld::StringMap(BTreeMap::from([(String::from(_name), to_ipld(value)?)]));
        Ok(())
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
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            Ipld::List(v) => {
                v.push(to_ipld(value)?);
                Ok(())
            }
            _ => {
                panic!("Expected List!");
            }
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            Ipld::StringMap(m) => {
                match &mut m.iter_mut().next() {
                    Some(entry) => match &mut entry.1 {
                        Ipld::List(v) => {
                            v.push(to_ipld(value)?);
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

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            Ipld::List(v) => {
                let mut pair = Vec::<Ipld>::with_capacity(2);
                pair.push(to_ipld(key)?);
                v.push(Ipld::List(pair));
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
        match &mut self.output {
            Ipld::List(v) => match &mut v.last_mut() {
                Some(Ipld::List(pair)) => {
                    pair.push(to_ipld(value)?);
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

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            Ipld::StringMap(m) => {
                m.insert(String::from(key), to_ipld(value)?);
            }
            _ => {
                panic!("Expected Map");
            }
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut IpldSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            Ipld::StringMap(top) => match &mut top.iter_mut().next() {
                Some(entry) => match &mut entry.1 {
                    Ipld::StringMap(m) => {
                        m.insert(String::from(key), to_ipld(value)?);
                    }
                    _ => {
                        panic!("Expected Map")
                    }
                },
                _ => {
                    panic!("Expected Non-Empty Map");
                }
            },
            _ => {
                panic!("Expected Map")
            }
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
