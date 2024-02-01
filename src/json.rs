use anyhow::Result;
use detect_indent::{detect_indent, Indent};
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;

pub fn deserialize<'a, T>(value: &'a str) -> Result<(T, Indent)>
where
    T: Deserialize<'a>,
{
    let json = serde_json::from_str::<T>(value)?;
    let indent = detect_indent(value);

    Ok((json, indent))
}

pub fn serialize<T>(value: &T, indent: &Indent) -> Result<String>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    let formatter = PrettyFormatter::with_indent(indent.indent().as_bytes());
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

    value.serialize(&mut serializer)?;
    let json = String::from_utf8(buf)?;

    Ok(json)
}
