use anyhow::Result;
use detect_indent::{detect_indent, Indent};
use detect_newline_style::LineEnding;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;

pub fn deserialize<'a, T>(value: &'a str) -> Result<(T, Indent, LineEnding)>
where
    T: Deserialize<'a>,
{
    let json = serde_json::from_str::<T>(value)?;
    let indent = detect_indent(value);
    let lineending = LineEnding::find_or_use_lf(value);

    Ok((json, indent, lineending))
}

pub fn serialize<T>(value: &T, indent: Indent, lineending: LineEnding) -> Result<String>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    let formatter = PrettyFormatter::with_indent(indent.indent().as_bytes());
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

    value.serialize(&mut serializer)?;
    let mut json = String::from_utf8(buf)?;
    json += match lineending {
        LineEnding::CR => "\r",
        LineEnding::LF => "\n",
        LineEnding::CRLF => "\r\n",
    };

    Ok(json)
}
