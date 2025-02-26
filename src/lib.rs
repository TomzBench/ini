/// Ini
mod parse;

#[cfg(feature = "serde")]
pub mod de;

use nom::Finish;
pub use parse::{Group, Key, Sections, Value};

/// Parse the ini format file
pub fn parse_str(input: &str) -> Result<Sections, parse::Error> {
    parse::tables(input).finish().map(|(_, sections)| sections)
}

#[cfg(feature = "serde")]
pub fn from_str<'a, T>(input: &'a str) -> Result<T, de::Error>
where
    T: serde::Deserialize<'a>,
{
    let mut deserializer = de::Deserializer::from_str(input);
    let t = T::deserialize(&mut deserializer)?;
    match deserializer.is_finished() {
        true => Ok(t),
        false => Err(de::Error::TrailingCharacters),
    }
}

#[cfg(feature = "serde")]
pub fn from_reader<R, T>(_reader: R) -> Result<T, de::Error>
where
    R: std::io::Read,
    T: serde::de::DeserializeOwned,
{
    unimplemented!()
}
