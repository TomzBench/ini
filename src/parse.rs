/// parse
use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::{
        complete::{alpha1, char, digit1, line_ending, multispace0, not_line_ending, space0},
        is_alphanumeric, is_space,
    },
    combinator::{map, map_res, opt, peek},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::{collections::HashMap, iter::FromIterator, str};

pub type Group<'a> = HashMap<Key<'a>, Value<'a>>;

pub type Sections<'a> = HashMap<&'a str, Group<'a>>;

pub type Error<'a> = nom::error::Error<&'a str>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    Num(i64),
    Str(&'a str),
    Array(Vec<Value<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key<'a> {
    Num(i64),
    Str(&'a str),
}

impl From<i64> for Key<'_> {
    fn from(value: i64) -> Self {
        Key::Num(value)
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(value: &'a str) -> Self {
        Key::Str(value)
    }
}

impl<'a> From<Key<'a>> for Value<'a> {
    fn from(value: Key<'a>) -> Self {
        match value {
            Key::Str(s) => Value::Str(s),
            Key::Num(n) => Value::Num(n),
        }
    }
}

pub(crate) fn eol(i: &str) -> IResult<&str, Option<&str>> {
    let (rest, (comment, _)) = pair(
        opt(preceded(pair(multispace0, char(';')), not_line_ending)),
        many1(line_ending),
    )(i)?;
    Ok((rest, comment))
}

// TODO allow underscore!
pub(crate) fn space_separated(i: &str) -> IResult<&str, &str> {
    let (i, result) = take_while(|c| is_alphanumeric(c as u8) || is_space(c as u8) || c == '_')(i)?;
    Ok((i, result.trim_end()))
}

pub(crate) fn key_like(i: &str) -> IResult<&str, Key> {
    preceded(
        multispace0,
        alt((
            map_res(digit1, |s| str::parse::<i64>(s).map(Key::Num)),
            map(space_separated, Key::Str),
        )),
    )(i)
}

pub(crate) fn values(i: &str) -> IResult<&str, Vec<Value>> {
    separated_list1(char(','), map(key_like, Value::from))(i)
}

pub(crate) fn value(i: &str) -> IResult<&str, Value> {
    match peek(terminated(key_like, char(',')))(i).is_ok() {
        true => map(values, Value::Array)(i),
        false => map(key_like, Value::from)(i),
    }
}

pub(crate) fn key_value(i: &str) -> IResult<&str, (Key, Value)> {
    pair(
        key_like,
        preceded(tuple((opt(space0), char('='), opt(space0))), value),
    )(i)
}

pub(crate) fn key_values(i: &str) -> IResult<&str, Group> {
    map(
        many0(terminated(key_value, opt(eol))),
        FromIterator::from_iter,
    )(i)
}

pub(crate) fn section(i: &str) -> IResult<&str, (&str, Option<&str>)> {
    delimited(
        char('['),
        pair(space_separated, opt(preceded(char(' '), alpha1))),
        char(']'),
    )(i)
}

pub(crate) fn group(i: &str) -> IResult<&str, (&str, Group)> {
    pair(
        map(terminated(section, opt(eol)), |(cat, _meta)| cat),
        key_values,
    )(i)
}

pub(crate) fn tables(i: &str) -> IResult<&str, Sections> {
    let (_i, (anon, mut named)): (_, (_, Sections)) =
        pair(opt(key_values), map(many0(group), FromIterator::from_iter))(i)?;
    if let Some(map) = anon {
        named.insert("_", map);
    }
    Ok((i, named))
}

#[cfg(feature = "serde")]
pub(crate) mod de {
    use super::{eol, key_like, section, Error, Key};
    use nom::{
        branch::alt,
        character::complete::{char, multispace0, space0},
        combinator::{map, opt, peek},
        sequence::{preceded, terminated, tuple},
        IResult,
    };
    use std::{error, fmt};

    #[derive(Debug)]
    pub struct OwnedError {
        pub code: nom::error::ErrorKind,
        pub input: String,
    }

    impl error::Error for OwnedError {}

    impl<'a> From<Error<'a>> for OwnedError {
        fn from(value: Error<'a>) -> Self {
            Self {
                code: value.code,
                input: value.input.to_string(),
            }
        }
    }

    impl fmt::Display for OwnedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "error {:?} at: {}", self.code, self.input)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum Ident<'a> {
        // TODO add a Subsection(&'a str) to enum to capture [.ident]
        Section(&'a str),
        Key(Key<'a>),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum OwnedKey {
        Num(i64),
        Str(String),
    }

    impl<'a> From<Key<'a>> for OwnedKey {
        fn from(value: Key<'a>) -> Self {
            match value {
                Key::Num(n) => Self::Num(n),
                Key::Str(s) => Self::Str(s.to_string()),
            }
        }
    }

    pub(crate) fn peek_ident(i: &str) -> Option<Ident> {
        peek(opt(ident))(i).map_or(None, |(_, ident)| ident)
    }

    pub(crate) fn peek_eof(i: &str) -> bool {
        peek(skip_end)(i).map_or_else(|_| false, |(rest, parsed)| rest.len() == parsed.len())
    }

    pub(crate) fn peek_eol(i: &str) -> bool {
        peek(eol)(i).map_or_else(|_| false, |_| true)
    }

    pub(crate) fn ident(i: &str) -> IResult<&str, Ident> {
        preceded(
            multispace0,
            alt((
                terminated(map(section, |(ident, _)| Ident::Section(ident)), opt(eol)),
                map(key_like, Ident::Key),
            )),
        )(i)
    }

    pub(crate) fn skip_end(i: &str) -> IResult<&str, &str> {
        multispace0(i)
    }

    pub(crate) fn assignment(i: &str) -> IResult<&str, (Option<&str>, char, Option<&str>)> {
        tuple((opt(space0), char('='), opt(space0)))(i)
    }

    pub(crate) fn comma(i: &str) -> IResult<&str, (Option<&str>, char, Option<&str>)> {
        tuple((opt(space0), char(','), opt(space0)))(i)
    }
}
