/// parse
use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{
        alpha1, char, digit1, line_ending, multispace0, not_line_ending, space0,
    },
    combinator::{map, map_res, opt},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated},
    AsChar, IResult, Parser,
};
use std::{borrow::Cow, collections::HashMap, fmt::Debug, iter::FromIterator, str};

pub type Group<'a> = HashMap<Key<'a>, Value<'a>>;

pub type Sections<'a> = HashMap<&'a str, Group<'a>>;

pub type Error<'a> = nom::error::Error<&'a str>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    Num(i64),
    Str(Cow<'a, str>),
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
            Key::Str(s) => Value::Str(Cow::Borrowed(s)),
            Key::Num(n) => Value::Num(n),
        }
    }
}

pub(crate) fn eol(i: &str) -> IResult<&str, Option<&str>> {
    let (rest, (comment, _)) = pair(
        opt(preceded(pair(multispace0, char(';')), not_line_ending)),
        many1(line_ending),
    )
    .parse(i)?;
    Ok((rest, comment))
}

pub(crate) fn backslash(i: &str) -> IResult<&str, char> {
    map(
        (multispace0, char('\\'), space0, line_ending, space0),
        |(_, slash, _, _, _)| slash,
    )
    .parse(i)
}

pub(crate) fn multiline(i: &str) -> IResult<&str, String> {
    map(
        preceded(backslash, separated_list1(backslash, space_separated)),
        |result| {
            result
                .into_iter()
                .map(|l| match l {
                    "" => "\n",
                    l => l,
                })
                .collect()
        },
    )
    .parse(i)
}

pub(crate) fn space_separated(i: &str) -> IResult<&str, &str> {
    take_while(|c| AsChar::is_alphanum(c as u8) || AsChar::is_space(c as u8) || c == '_')(i)
}

pub(crate) fn key_like(i: &str) -> IResult<&str, Key> {
    preceded(
        multispace0,
        alt((
            map_res(digit1, |s| str::parse::<i64>(s).map(Key::Num)),
            map(space_separated, |val| Key::Str(val.trim_end())),
        )),
    )
    .parse(i)
}

pub(crate) fn values(i: &str) -> IResult<&str, Vec<Value>> {
    let (i, vec) = separated_list1(char(','), map(key_like, Value::from)).parse(i)?;
    match vec.len() {
        0..2 => Err(nom::Err::Error(nom::error::Error::from_error_kind(
            i,
            nom::error::ErrorKind::SeparatedNonEmptyList,
        ))),
        2.. => Ok((i, vec)),
    }
}

pub(crate) fn value(i: &str) -> IResult<&str, Value> {
    alt((
        terminated(map(multiline, |s| Value::Str(Cow::Owned(s))), eol),
        terminated(map(values, Value::Array), eol),
        terminated(map(key_like, Value::from), eol),
    ))
    .parse(i)
}

pub(crate) fn key_value(i: &str) -> IResult<&str, (Key, Value)> {
    pair(
        key_like,
        preceded((opt(space0), char('='), opt(space0)), value),
    )
    .parse(i)
}

pub(crate) fn key_values(i: &str) -> IResult<&str, Group> {
    map(many0(key_value), FromIterator::from_iter).parse(i)
}

pub(crate) fn section(i: &str) -> IResult<&str, (&str, Option<&str>)> {
    terminated(
        delimited(
            char('['),
            pair(space_separated, opt(preceded(char(' '), alpha1))),
            char(']'),
        ),
        eol,
    )
    .parse(i)
}

pub(crate) fn group(i: &str) -> IResult<&str, (&str, Group)> {
    pair(map(section, |(cat, _meta)| cat), key_values).parse(i)
}

pub(crate) fn tables(i: &str) -> IResult<&str, Sections> {
    let (i, (anon, mut named)): (_, (_, Sections)) =
        pair(opt(key_values), map(many0(group), FromIterator::from_iter)).parse(i)?;
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
        sequence::{preceded, terminated},
        IResult, Parser,
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
        peek(opt(ident)).parse(i).map_or(None, |(_, ident)| ident)
    }

    pub(crate) fn peek_eof(i: &str) -> bool {
        peek(skip_end)
            .parse(i)
            .map_or_else(|_| false, |(rest, parsed)| rest.len() == parsed.len())
    }

    pub(crate) fn peek_eol(i: &str) -> bool {
        peek(eol).parse(i).map_or_else(|_| false, |_| true)
    }

    pub(crate) fn ident(i: &str) -> IResult<&str, Ident> {
        preceded(
            multispace0,
            alt((
                terminated(map(section, |(ident, _)| Ident::Section(ident)), opt(eol)),
                map(key_like, Ident::Key),
            )),
        )
        .parse(i)
    }

    pub(crate) fn skip_end(i: &str) -> IResult<&str, &str> {
        multispace0(i)
    }

    pub(crate) fn assignment(i: &str) -> IResult<&str, (Option<&str>, char, Option<&str>)> {
        (opt(space0), char('='), opt(space0)).parse(i)
    }

    pub(crate) fn comma(i: &str) -> IResult<&str, (Option<&str>, char, Option<&str>)> {
        (opt(space0), char(','), opt(space0)).parse(i)
    }
}
