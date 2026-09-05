//! EOL normalization actions. Whole-file, no options. Mirrors .NET `EolActions.cs`.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;

use crate::model::Step;

fn convert(input: &str, to: &str) -> Result<String> {
    let re = Regex::new(r"\r\n|\r|\n").map_err(|e| anyhow!(e))?;
    Ok(re.replace_all(input, to).into_owned())
}

/// `eol_crlf` / `eol_windows`.
pub fn to_crlf(input: &str, _step: &Step) -> Result<String> {
    convert(input, "\r\n")
}

/// `eol_lf` / `eol_unix`.
pub fn to_lf(input: &str, _step: &Step) -> Result<String> {
    convert(input, "\n")
}

/// `eol_cr` / `eol_mac`.
pub fn to_cr(input: &str, _step: &Step) -> Result<String> {
    convert(input, "\r")
}
