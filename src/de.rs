/// de
use crate::{parse, Key};
use nom::Finish;
use serde::de::{self, IntoDeserializer, SeqAccess};
use std::{error, fmt, num::ParseIntError};

#[derive(Debug)]
pub enum Error {
    Parser(parse::de::OwnedError),
    Message(String),
    TrailingCharacters,
    ExpectAssignment(parse::de::OwnedError),
    ExpectBool(parse::de::OwnedKey),
    ExpectChar(parse::de::OwnedKey),
    ExpectNum(ParseIntError),
    ExpectIdent(String),
    Unsupported(&'static str),
}

impl<'a> error::Error for Error {}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl<'a> From<parse::Error<'a>> for Error {
    fn from(value: parse::Error<'a>) -> Self {
        Error::Parser(value.into())
    }
}

impl<'a> From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::ExpectNum(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => fmt.write_str(msg),
            Error::TrailingCharacters => write!(fmt, "junk at end of input"),
            Error::Parser(e) => e.fmt(fmt),
            Error::ExpectAssignment(e) => write!(fmt, "expected assignment, found {e}"),
            Error::ExpectBool(key) => write!(fmt, "expected bool, found {:?}", key),
            Error::ExpectChar(key) => write!(fmt, "expected char, found {:?}", key),
            Error::ExpectNum(key) => write!(fmt, "expected number, found {:?}", key),
            Error::ExpectIdent(key) => write!(fmt, "expected number, found {:?}", key),
            Error::Unsupported(s) => write!(fmt, "{s} is not unsupported"),
        }
    }
}

#[derive(Debug)]
pub struct Deserializer<'de> {
    // Remaining input string left to be parsed
    input: &'de str,
    // A cached identifier for validating subsection names
    ident: Option<&'de str>,
}

impl<'de> Deserializer<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Self {
        Self { input, ident: None }
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.input.is_empty()
    }

    fn peek_ident(&self) -> Option<parse::de::Ident> {
        parse::de::peek_ident(self.input)
    }

    fn parse_key_like(&mut self) -> Result<parse::Key<'de>, Error> {
        let (input, key) = parse::key_like(self.input).finish()?;
        self.input = input;
        Ok(key)
    }

    fn parse_ident(&mut self) -> Result<parse::de::Ident<'de>, Error> {
        let (input, ident) = parse::de::ident(self.input).finish()?;
        self.input = input;
        Ok(ident)
    }

    fn parse_assignment(&mut self) -> Result<char, Error> {
        let (input, (_, c, _)) = parse::de::assignment(self.input).finish()?;
        self.input = input;
        Ok(c)
    }

    fn parse_comma(&mut self) -> Result<char, Error> {
        let (input, (_, c, _)) = parse::de::comma(self.input).finish()?;
        self.input = input;
        Ok(c)
    }

    fn check_eol(&mut self) -> bool {
        if parse::de::peek_eol(self.input) {
            match parse::eol(self.input).finish() {
                Err(_) => unreachable!(), // We peeked eol so we know its guarentee Ok
                Ok((input, _)) => {
                    self.input = input;
                    true
                }
            }
        } else {
            false
        }
    }

    fn check_eof(&mut self) -> bool {
        if parse::de::peek_eof(self.input) {
            self.done();
            true
        } else {
            false
        }
    }

    fn done(&mut self) {
        self.input = "";
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Num(n) => visitor.visit_i64(n),
            Key::Str(s) => visitor.visit_i64(s.parse::<i64>()?),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Num(n) => visitor.visit_u64(n as u64),
            Key::Str(s) => visitor.visit_u64(s.parse::<u64>()?),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported("f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported("f64"))
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported("bytes"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported("byte buf"))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Str(v) => visitor.visit_borrowed_str(v),
            Key::Num(v) => visitor.visit_str(v.to_string().as_str()),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Str(v) => match v.chars().next() {
                Some(c) => visitor.visit_char(c),
                None => Err(Error::ExpectChar(value.into())),
            },
            val => Err(Error::ExpectChar(val.into())),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Str("true") => visitor.visit_bool(true),
            Key::Str("True") => visitor.visit_bool(true),
            Key::Str("TRUE") => visitor.visit_bool(true),
            Key::Str("false") => visitor.visit_bool(false),
            Key::Str("False") => visitor.visit_bool(false),
            Key::Str("FALSE") => visitor.visit_bool(false),
            Key::Num(0) => visitor.visit_bool(false),
            Key::Num(1) => visitor.visit_bool(true),
            e => Err(Error::ExpectBool(e.into())),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.parse_key_like()?;
        match value {
            Key::Str(s) => visitor.visit_enum(s.into_deserializer()),
            Key::Num(n) => visitor.visit_enum((n as u32).into_deserializer()),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let ident = self.ident;
        let value = visitor.visit_map(MapAccess { de: self, ident })?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(Sequence {
            de: self,
            first: true,
        })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let ident = self.parse_ident()?;
        match ident {
            parse::de::Ident::Key(Key::Num(n)) => visitor.visit_i64(n),
            parse::de::Ident::Key(Key::Str(s)) => visitor.visit_borrowed_str(s),
            parse::de::Ident::Section(key) => {
                self.ident = Some(key);
                visitor.visit_borrowed_str(key)
            }
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = self.parse_key_like()?;
        match v {
            Key::Str(v) => visitor.visit_borrowed_str(v),
            Key::Num(v) => visitor.visit_i64(v),
        }
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    ident: Option<&'de str>,
}

impl<'de, 'a> de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.check_eof() {
            true => Ok(None),
            false => {
                // TODO when we support subsections, we can compare next ident to [ident.subsection]
                match self.de.peek_ident() {
                    // A nested [section] is detected. Close this map
                    Some(parse::de::Ident::Section(_ident)) if self.ident.is_some() => Ok(None),
                    // A root [section] is detected.
                    Some(parse::de::Ident::Section(_ident)) => {
                        seed.deserialize(&mut *self.de).map(Some)
                    }
                    // A normal key, value pair is detected
                    Some(parse::de::Ident::Key(_ident)) => {
                        let result = seed.deserialize(&mut *self.de).map(Some)?;
                        self.de.parse_assignment()?;
                        Ok(result)
                    }
                    None => Err(Error::ExpectIdent(self.de.input.to_string())),
                }
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let result = seed.deserialize(&mut *self.de)?;
        self.de.check_eol();
        Ok(result)
    }
}

struct Sequence<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de, 'a> SeqAccess<'de> for Sequence<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.check_eol() || self.de.check_eof() {
            Ok(None)
        } else if let Some(parse::de::Ident::Section(_)) = self.de.peek_ident() {
            Ok(None)
        } else {
            if !self.first {
                self.de.parse_comma()?;
            }
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}
