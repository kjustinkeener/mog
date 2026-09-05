//! Encoding helpers that operate on the whole document text: percent-encoding,
//! HTML entity encoding, and base64.

use anyhow::{anyhow, bail, Result};
use base64::Engine;

use crate::line_text::{join, split};
use crate::model::Step;

/// `url_encode`: percent-encode the whole text (RFC 3986 unreserved set is left
/// as-is; everything else becomes `%XX`).
pub fn url_encode(input: &str, _step: &Step) -> Result<String> {
    Ok(urlencoding::encode(input).into_owned())
}

/// `url_decode`: reverse percent-encoding. Errors if the result is not UTF-8.
pub fn url_decode(input: &str, _step: &Step) -> Result<String> {
    urlencoding::decode(input)
        .map(|c| c.into_owned())
        .map_err(|e| anyhow!("invalid percent-encoding: {e}"))
}

/// `html_encode`: encode the five HTML-significant characters `& < > " '`.
pub fn html_encode(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    Ok(out)
}

/// Decode a named HTML entity (the text between `&` and `;`) to its string, or
/// None for an unknown name. A common subset; numeric entities (`&#NN;`, `&#xHH;`)
/// cover everything else. Non-ASCII values are written as `\u{..}` escapes so the
/// source stays plain ASCII.
fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{a0}",
        "copy" => "\u{a9}",
        "reg" => "\u{ae}",
        "trade" => "\u{2122}",
        "hellip" => "\u{2026}",
        "mdash" => "\u{2014}",
        "ndash" => "\u{2013}",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "ldquo" => "\u{201c}",
        "rdquo" => "\u{201d}",
        "bull" => "\u{2022}",
        "middot" => "\u{b7}",
        "deg" => "\u{b0}",
        "plusmn" => "\u{b1}",
        "times" => "\u{d7}",
        "divide" => "\u{f7}",
        "frac12" => "\u{bd}",
        "frac14" => "\u{bc}",
        "frac34" => "\u{be}",
        "euro" => "\u{20ac}",
        "pound" => "\u{a3}",
        "cent" => "\u{a2}",
        "yen" => "\u{a5}",
        "sect" => "\u{a7}",
        "para" => "\u{b6}",
        "laquo" => "\u{ab}",
        "raquo" => "\u{bb}",
        "dagger" => "\u{2020}",
        "Dagger" => "\u{2021}",
        "permil" => "\u{2030}",
        "prime" => "\u{2032}",
        "Prime" => "\u{2033}",
        "larr" => "\u{2190}",
        "rarr" => "\u{2192}",
        "uarr" => "\u{2191}",
        "darr" => "\u{2193}",
        "harr" => "\u{2194}",
        "eacute" => "\u{e9}",
        "egrave" => "\u{e8}",
        "agrave" => "\u{e0}",
        "ccedil" => "\u{e7}",
        "auml" => "\u{e4}",
        "ouml" => "\u{f6}",
        "uuml" => "\u{fc}",
        "szlig" => "\u{df}",
        "ntilde" => "\u{f1}",
        "Ntilde" => "\u{d1}",
        _ => return None,
    })
}

/// `html_decode`: reverse HTML entity encoding. Handles named entities (a common
/// subset plus the `html_encode` set), decimal numeric entities (`&#NN;`), and hex
/// numeric entities (`&#xHH;`), in a single left-to-right pass so double-encoded
/// sequences (e.g. `&amp;lt;` -> `&lt;`) resolve one level. An unknown `&name;` is
/// left verbatim rather than dropped.
pub fn html_decode(input: &str, _step: &Step) -> Result<String> {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if bytes[i] == b'&' {
            if let Some(semi_rel) = input[i + 1..].find(';') {
                // A bounded body between & and ; ; longer runs are not an entity.
                if (1..=32).contains(&semi_rel) {
                    let body = &input[i + 1..i + 1 + semi_rel];
                    let decoded: Option<String> = if let Some(hex) =
                        body.strip_prefix("#x").or_else(|| body.strip_prefix("#X"))
                    {
                        u32::from_str_radix(hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| c.to_string())
                    } else if let Some(dec) = body.strip_prefix('#') {
                        dec.parse::<u32>()
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| c.to_string())
                    } else {
                        named_entity(body).map(|s| s.to_string())
                    };
                    if let Some(d) = decoded {
                        out.push_str(&d);
                        i = i + 1 + semi_rel + 1;
                        continue;
                    }
                }
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

/// `base64_encode`: standard base64 of the whole text (UTF-8 bytes).
pub fn base64_encode(input: &str, _step: &Step) -> Result<String> {
    Ok(base64::engine::general_purpose::STANDARD.encode(input.as_bytes()))
}

/// `base64_decode`: decode standard base64 back to text. Surrounding whitespace
/// (e.g. a trailing newline from a file) is ignored. Errors on invalid base64 or
/// non-UTF-8 output.
pub fn base64_decode(input: &str, _step: &Step) -> Result<String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(input.trim().as_bytes())
        .map_err(|e| anyhow!("invalid base64: {e}"))?;
    String::from_utf8(bytes).map_err(|e| anyhow!("base64 decoded to invalid UTF-8: {e}"))
}

/// `rot13`: rotate each ASCII letter by 13 places (A-Z and a-z), leaving every
/// other byte untouched. Self-inverse: applying it twice restores the original.
pub fn rot13(input: &str, _step: &Step) -> Result<String> {
    let out: String = input
        .chars()
        .map(|c| match c {
            'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
            'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
            _ => c,
        })
        .collect();
    Ok(out)
}

/// `hex_encode`: hexlify the whole text -- each UTF-8 byte becomes two lowercase
/// hex digits, with no separators. The exact inverse of `hex_decode`.
pub fn hex_encode(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.as_bytes() {
        out.push_str(&format!("{b:02x}"));
    }
    Ok(out)
}

/// `hex_decode`: unhexlify a lowercase/uppercase hex string back to text.
/// Surrounding whitespace (e.g. a trailing newline from a file) is ignored.
/// Errors cleanly on an odd number of hex digits, a non-hex character, or output
/// that is not valid UTF-8.
pub fn hex_decode(input: &str, _step: &Step) -> Result<String> {
    let trimmed = input.trim();
    if !trimmed.len().is_multiple_of(2) {
        bail!(
            "invalid hex: odd number of digits ({} chars)",
            trimmed.len()
        );
    }
    let bytes = trimmed.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    String::from_utf8(out).map_err(|e| anyhow!("hex decoded to invalid UTF-8: {e}"))
}

/// One hex digit's value, or a clean error naming the offending character.
fn hex_val(b: u8) -> Result<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        other => Err(anyhow!(
            "invalid hex: '{}' is not a hex digit",
            other as char
        )),
    }
}

/// `normalize_url`: canonicalize each line that is an http(s)-style URL, leaving
/// any non-URL line untouched. Lowercases the scheme and host, removes a default
/// port (80 for http, 443 for https), drops a trailing dot on the host, and drops
/// an empty `?` query. Options: `lowercase_host` (default true),
/// `remove_default_port` (default true), `sort_query` (sort `&`-separated query
/// params, default false), `strip_fragment` (drop `#...`, default false).
/// Best-effort for hierarchical `scheme://` URLs; it does not re-encode paths.
pub fn normalize_url(input: &str, step: &Step) -> Result<String> {
    let opts = UrlOpts {
        lowercase_host: step.get_bool("lowercase_host", true)?,
        remove_default_port: step.get_bool("remove_default_port", true)?,
        sort_query: step.get_bool("sort_query", false)?,
        strip_fragment: step.get_bool("strip_fragment", false)?,
    };
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| normalize_one_url(l, &opts))
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

struct UrlOpts {
    lowercase_host: bool,
    remove_default_port: bool,
    sort_query: bool,
    strip_fragment: bool,
}

fn normalize_one_url(line: &str, opts: &UrlOpts) -> String {
    // Require a `scheme://` prefix with a plausible scheme; otherwise pass through.
    let Some(scheme_end) = line.find("://") else {
        return line.to_string();
    };
    let scheme = &line[..scheme_end];
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return line.to_string();
    }
    let scheme_lc = scheme.to_ascii_lowercase();
    let after = &line[scheme_end + 3..];
    let auth_end = after.find(['/', '?', '#']).unwrap_or(after.len());
    let authority = &after[..auth_end];
    let rest = &after[auth_end..];

    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((u, h)) => (Some(u), h),
        None => (None, authority),
    };
    let (host, port) = split_host_port(hostport);
    let mut host_norm = if opts.lowercase_host {
        host.to_ascii_lowercase()
    } else {
        host.to_string()
    };
    if host_norm.ends_with('.') {
        host_norm.pop();
    }
    let port_norm = port.filter(|p| {
        !(opts.remove_default_port
            && ((scheme_lc == "http" && *p == "80") || (scheme_lc == "https" && *p == "443")))
    });

    let mut auth_out = String::new();
    if let Some(u) = userinfo {
        auth_out.push_str(u);
        auth_out.push('@');
    }
    auth_out.push_str(&host_norm);
    if let Some(p) = port_norm {
        auth_out.push(':');
        auth_out.push_str(p);
    }

    let (before_frag, fragment) = match rest.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (rest, None),
    };
    let (path, query) = match before_frag.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (before_frag, None),
    };

    let mut out = format!("{scheme_lc}://{auth_out}{path}");
    if let Some(q) = query {
        if !q.is_empty() {
            let q2 = if opts.sort_query {
                let mut parts: Vec<&str> = q.split('&').collect();
                parts.sort_unstable();
                parts.join("&")
            } else {
                q.to_string()
            };
            out.push('?');
            out.push_str(&q2);
        }
    }
    if !opts.strip_fragment {
        if let Some(f) = fragment {
            out.push('#');
            out.push_str(f);
        }
    }
    out
}

/// Split an authority's `host[:port]`, honoring a bracketed IPv6 literal. A
/// non-numeric or empty port is treated as part of the host (left as-is).
fn split_host_port(hp: &str) -> (&str, Option<&str>) {
    if hp.starts_with('[') {
        if let Some(end) = hp.find(']') {
            let host = &hp[..=end];
            let port = hp[end + 1..].strip_prefix(':').filter(|p| !p.is_empty());
            return (host, port);
        }
    }
    match hp.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => (h, Some(p)),
        _ => (hp, None),
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn normalize_lowercases_and_drops_default_port() {
        let out = run(
            r#"{"steps":[{"action":"normalize_url"}]}"#,
            "HTTP://Example.COM:80/Path?b=2",
        );
        assert_eq!(out, "http://example.com/Path?b=2");
    }

    #[test]
    fn normalize_sorts_query_and_drops_trailing_dot() {
        let out = run(
            r#"{"steps":[{"action":"normalize_url","options":{"sort_query":true}}]}"#,
            "https://Host.:443/x?b=2&a=1#f",
        );
        assert_eq!(out, "https://host/x?a=1&b=2#f");
    }

    #[test]
    fn normalize_can_strip_fragment_and_empty_query() {
        let out = run(
            r#"{"steps":[{"action":"normalize_url","options":{"strip_fragment":true}}]}"#,
            "https://host/x?#frag",
        );
        assert_eq!(out, "https://host/x");
    }

    #[test]
    fn normalize_leaves_non_url_lines() {
        let out = run(
            r#"{"steps":[{"action":"normalize_url"}]}"#,
            "just some text\nmailto:a@b.com",
        );
        assert_eq!(out, "just some text\nmailto:a@b.com");
    }

    #[test]
    fn normalize_preserves_ipv6_host_and_userinfo() {
        let out = run(
            r#"{"steps":[{"action":"normalize_url"}]}"#,
            "http://User@[2001:DB8::1]:80/p",
        );
        // Host (incl. IPv6 hex) is lowercased; userinfo is preserved.
        assert_eq!(out, "http://User@[2001:db8::1]/p");
    }

    #[test]
    fn html_decode_named_numeric_and_hex() {
        let out = run(
            r#"{"steps":[{"action":"html_decode"}]}"#,
            "a &lt;b&gt; &amp; &#39;c&#39; &#x41; &nbsp;end &copy;",
        );
        assert_eq!(out, "a <b> & 'c' A \u{a0}end \u{a9}");
    }

    #[test]
    fn html_decode_leaves_unknown_entities_verbatim() {
        let out = run(
            r#"{"steps":[{"action":"html_decode"}]}"#,
            "keep &notareal; and A & B",
        );
        assert_eq!(out, "keep &notareal; and A & B");
    }

    #[test]
    fn html_decode_resolves_one_level_of_double_encoding() {
        let out = run(r#"{"steps":[{"action":"html_decode"}]}"#, "&amp;lt;");
        assert_eq!(out, "&lt;");
    }

    #[test]
    fn rot13_is_self_inverse_and_ignores_non_letters() {
        let src = "Hello, World! 123";
        let enc = run(r#"{"steps":[{"action":"rot13"}]}"#, src);
        assert_eq!(enc, "Uryyb, Jbeyq! 123");
        // Applying rot13 again restores the original.
        let dec = run(r#"{"steps":[{"action":"rot13"}]}"#, &enc);
        assert_eq!(dec, src);
    }

    #[test]
    fn hex_encode_lowercase() {
        let out = run(r#"{"steps":[{"action":"hex_encode"}]}"#, "Hi!");
        assert_eq!(out, "486921");
    }

    #[test]
    fn hex_round_trips() {
        let src = "mog rocks";
        let enc = run(r#"{"steps":[{"action":"hex_encode"}]}"#, src);
        let dec = run(r#"{"steps":[{"action":"hex_decode"}]}"#, &enc);
        assert_eq!(dec, src);
    }

    #[test]
    fn hex_decode_rejects_odd_length() {
        let e = crate::execute(
            &parse_mog(r#"{"steps":[{"action":"hex_decode"}]}"#).unwrap(),
            "abc",
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("odd number of digits"), "got: {e}");
    }

    #[test]
    fn hex_decode_rejects_non_hex() {
        let e = crate::execute(
            &parse_mog(r#"{"steps":[{"action":"hex_decode"}]}"#).unwrap(),
            "zz",
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("not a hex digit"), "got: {e}");
    }

    #[test]
    fn html_encode_then_decode_round_trips() {
        let src = "Tom & Jerry <say> \"hi\" 'yo'";
        let enc = run(r#"{"steps":[{"action":"html_encode"}]}"#, src);
        let dec = run(r#"{"steps":[{"action":"html_decode"}]}"#, &enc);
        assert_eq!(dec, src);
    }
}
