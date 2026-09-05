//! Input decoding and output encoding at the CLI's byte boundary.
//!
//! The pipeline itself operates on UTF-8 `String`s, so character encoding is an
//! I/O concern, not a text transform: bytes are decoded to a `String` on the way
//! in and encoded back to bytes on the way out. This mirrors a text editor's
//! "Encoding" menu (auto-detect on open, "Convert to ..." on save).
//!
//! Input (`--encoding`): `auto` (default) sniffs a BOM (UTF-8, UTF-16 LE/BE),
//! else tries strict UTF-8, else falls back to Windows-1252 ("ANSI"). Any
//! explicit label understood by the WHATWG Encoding Standard is also accepted
//! (`utf-16le`, `windows-1252`, `shift_jis`, `iso-8859-1`, ...), plus the alias
//! `ansi` for Windows-1252.
//!
//! Output (`--output-encoding`): `preserve` (default) round-trips the detected
//! input encoding and BOM state; or force `utf-8`, `utf-8-bom`, `utf-16le`,
//! `utf-16be`, `ansi`/`windows-1252`, or any other single/multi-byte label. Note
//! UTF-16 is written by hand here because `encoding_rs` decodes but does not
//! encode UTF-16.

use anyhow::{anyhow, Result};
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};

/// The result of decoding input bytes to text, plus what it was decoded as (so
/// output can `preserve` it).
#[derive(Debug, Clone)]
pub struct Decoded {
    pub text: String,
    /// Canonical encoding name (e.g. "UTF-8", "UTF-16LE", "windows-1252").
    pub encoding: String,
    pub had_bom: bool,
}

/// Decode `bytes` to text. `requested` is a `--encoding` label, or `None`/`auto`
/// for BOM-sniffing auto-detection. `lossy` allows replacement of malformed
/// sequences instead of erroring (only meaningful for a strict UTF-8 request).
pub fn decode(bytes: &[u8], requested: Option<&str>, lossy: bool) -> Result<Decoded> {
    match requested {
        None => decode_auto(bytes),
        Some(name) if name.eq_ignore_ascii_case("auto") => decode_auto(bytes),
        Some(name) => decode_labeled(bytes, name, lossy),
    }
}

fn decode_auto(bytes: &[u8]) -> Result<Decoded> {
    if let Some((enc, bom_len)) = Encoding::for_bom(bytes) {
        let (cow, _) = enc.decode_without_bom_handling(&bytes[bom_len..]);
        return Ok(Decoded {
            text: cow.into_owned(),
            encoding: enc.name().to_string(),
            had_bom: true,
        });
    }
    // No BOM: prefer strict UTF-8, else fall back to Windows-1252 ("ANSI"),
    // which maps every byte so it never fails.
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(Decoded {
            text: s.to_string(),
            encoding: UTF_8.name().to_string(),
            had_bom: false,
        }),
        Err(_) => {
            let (cow, _) = WINDOWS_1252.decode_without_bom_handling(bytes);
            Ok(Decoded {
                text: cow.into_owned(),
                encoding: WINDOWS_1252.name().to_string(),
                had_bom: false,
            })
        }
    }
}

fn decode_labeled(bytes: &[u8], name: &str, lossy: bool) -> Result<Decoded> {
    let enc = resolve_encoding(name)?;
    let had_bom = matches!(Encoding::for_bom(bytes), Some((e, _)) if e == enc);

    if enc == UTF_8 && !lossy {
        // Strict UTF-8: strip a UTF-8 BOM, then validate.
        let payload = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
        let text = std::str::from_utf8(payload)
            .map_err(|e| anyhow!("input is not valid UTF-8 ({e}); try --encoding auto or --lossy"))?
            .to_string();
        return Ok(Decoded {
            text,
            encoding: UTF_8.name().to_string(),
            had_bom,
        });
    }

    let (cow, _) = enc.decode_with_bom_removal(bytes);
    Ok(Decoded {
        text: cow.into_owned(),
        encoding: enc.name().to_string(),
        had_bom,
    })
}

/// Map a user label to an encoding, accepting the `ansi` alias for Windows-1252.
fn resolve_encoding(name: &str) -> Result<&'static Encoding> {
    let n = name.trim();
    if n.eq_ignore_ascii_case("ansi") {
        return Ok(WINDOWS_1252);
    }
    Encoding::for_label(n.as_bytes()).ok_or_else(|| anyhow!("unknown encoding '{name}'"))
}

/// Encode pipeline output text to bytes for a `--output-encoding` choice.
/// `detected` supports the default `preserve` mode.
pub fn encode(text: &str, choice: &str, detected: &Decoded) -> Result<Vec<u8>> {
    let c = choice.trim().to_ascii_lowercase();
    match c.as_str() {
        "preserve" => Ok(encode_named(text, &detected.encoding, detected.had_bom)),
        "utf-8" | "utf8" => Ok(text.as_bytes().to_vec()),
        "utf-8-bom" | "utf8-bom" => {
            let mut v = vec![0xEF, 0xBB, 0xBF];
            v.extend_from_slice(text.as_bytes());
            Ok(v)
        }
        "utf-16" | "utf-16le" | "utf-16le-bom" => Ok(utf16(text, true, true)),
        "utf-16be" | "utf-16be-bom" => Ok(utf16(text, false, true)),
        "ansi" | "windows-1252" => Ok(WINDOWS_1252.encode(text).0.into_owned()),
        other => {
            let enc = resolve_encoding(other)?;
            // encoding_rs cannot encode to UTF-16; those are handled above.
            Ok(enc.encode(text).0.into_owned())
        }
    }
}

/// Encode `text` as the named encoding (used by `preserve`).
fn encode_named(text: &str, name: &str, had_bom: bool) -> Vec<u8> {
    match name {
        "UTF-16LE" => utf16(text, true, had_bom),
        "UTF-16BE" => utf16(text, false, had_bom),
        "UTF-8" => {
            if had_bom {
                let mut v = vec![0xEF, 0xBB, 0xBF];
                v.extend_from_slice(text.as_bytes());
                v
            } else {
                text.as_bytes().to_vec()
            }
        }
        other => Encoding::for_label(other.as_bytes())
            .unwrap_or(WINDOWS_1252)
            .encode(text)
            .0
            .into_owned(),
    }
}

/// Hand-rolled UTF-16 encoder (`encoding_rs` is decode-only for UTF-16).
fn utf16(text: &str, little_endian: bool, bom: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 2 + 2);
    if bom {
        out.extend_from_slice(if little_endian {
            &[0xFF, 0xFE]
        } else {
            &[0xFE, 0xFF]
        });
    }
    for unit in text.encode_utf16() {
        let bytes = if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        };
        out.extend_from_slice(&bytes);
    }
    out
}

/// Validate a `--output-encoding` label up front (so a bad value fails fast,
/// before any file work). Returns the normalized label for reuse.
pub fn validate_output_choice(choice: &str) -> Result<()> {
    let c = choice.trim().to_ascii_lowercase();
    match c.as_str() {
        "preserve" | "utf-8" | "utf8" | "utf-8-bom" | "utf8-bom" | "utf-16" | "utf-16le"
        | "utf-16le-bom" | "utf-16be" | "utf-16be-bom" | "ansi" | "windows-1252" => Ok(()),
        other => resolve_encoding(other).map(|_| ()).map_err(|_| {
            anyhow!("unknown --output-encoding '{choice}' (try utf-8, utf-8-bom, utf-16le, utf-16be, ansi, or a WHATWG label)")
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_utf8_no_bom() {
        let d = decode("héllo".as_bytes(), None, false).unwrap();
        assert_eq!(d.text, "héllo");
        assert_eq!(d.encoding, "UTF-8");
        assert!(!d.had_bom);
    }

    #[test]
    fn auto_utf8_bom() {
        let mut b = vec![0xEF, 0xBB, 0xBF];
        b.extend_from_slice("hi".as_bytes());
        let d = decode(&b, None, false).unwrap();
        assert_eq!(d.text, "hi");
        assert_eq!(d.encoding, "UTF-8");
        assert!(d.had_bom);
    }

    #[test]
    fn auto_utf16le_bom() {
        let b = utf16("SELECT 1", true, true);
        let d = decode(&b, None, false).unwrap();
        assert_eq!(d.text, "SELECT 1");
        assert_eq!(d.encoding, "UTF-16LE");
        assert!(d.had_bom);
    }

    #[test]
    fn auto_utf16be_bom() {
        let b = utf16("café", false, true);
        let d = decode(&b, None, false).unwrap();
        assert_eq!(d.text, "café");
        assert_eq!(d.encoding, "UTF-16BE");
    }

    #[test]
    fn auto_falls_back_to_ansi() {
        // 0xE9 is 'é' in Windows-1252 but invalid as lone UTF-8.
        let d = decode(&[b'c', b'a', b'f', 0xE9], None, false).unwrap();
        assert_eq!(d.text, "café");
        assert_eq!(d.encoding, "windows-1252");
    }

    #[test]
    fn strict_utf8_request_errors_on_bad_bytes() {
        assert!(decode(&[0xE9, 0xFF], Some("utf-8"), false).is_err());
        // ...but lossy replaces instead.
        assert!(decode(&[0xE9, 0xFF], Some("utf-8"), true).is_ok());
    }

    #[test]
    fn roundtrip_preserve_utf16le() {
        let original = utf16("Grüße 世界", true, true);
        let d = decode(&original, None, false).unwrap();
        let out = encode(&d.text, "preserve", &d).unwrap();
        assert_eq!(out, original);
    }

    #[test]
    fn convert_utf16_input_to_utf8() {
        let src = utf16("CREATE TABLE t", true, true);
        let d = decode(&src, None, false).unwrap();
        let out = encode(&d.text, "utf-8", &d).unwrap();
        assert_eq!(out, "CREATE TABLE t".as_bytes());
    }

    #[test]
    fn convert_to_utf8_bom() {
        let d = decode("x".as_bytes(), None, false).unwrap();
        assert_eq!(
            encode(&d.text, "utf-8-bom", &d).unwrap(),
            vec![0xEF, 0xBB, 0xBF, b'x']
        );
    }

    #[test]
    fn ansi_alias_resolves() {
        assert!(resolve_encoding("ansi").is_ok());
        assert!(validate_output_choice("ansi").is_ok());
        assert!(validate_output_choice("nonsense-enc").is_err());
    }
}
