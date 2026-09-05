//! EOL-aware line splitting/joining that preserves the dominant EOL style and
//! the trailing-newline state, mirroring the .NET `LineText.cs` helper.

/// Detect the dominant EOL by scanning for the first line terminator.
/// Returns "\r\n", "\r", or "\n" (default "\n" when none is found).
pub fn detect_eol(text: &str) -> &'static str {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    return "\r\n";
                }
                return "\r";
            }
            b'\n' => return "\n",
            _ => {}
        }
        i += 1;
    }
    "\n"
}

/// Split result: the lines (without terminators), the detected EOL, and
/// whether the text ended with a terminator.
pub struct SplitText {
    pub lines: Vec<String>,
    pub eol: &'static str,
    pub trailing_eol: bool,
}

/// Split on any of \r\n, \r, \n (mixed endings supported), dropping the single
/// phantom empty element produced after a trailing terminator.
pub fn split(text: &str) -> SplitText {
    let eol = detect_eol(text);
    if text.is_empty() {
        return SplitText {
            lines: Vec::new(),
            eol,
            trailing_eol: false,
        };
    }

    let last = text.as_bytes()[text.len() - 1];
    let trailing_eol = last == b'\n' || last == b'\r';

    let mut lines = split_any_eol(text);

    // Phantom-line fix: remove only ONE trailing empty element.
    if trailing_eol {
        if let Some(last_line) = lines.last() {
            if last_line.is_empty() {
                lines.pop();
            }
        }
    }

    SplitText {
        lines,
        eol,
        trailing_eol,
    }
}

/// Equivalent of .NET `Regex.Split(text, "\r\n|\r|\n")`.
fn split_any_eol(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                out.push(text[start..i].to_string());
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    i += 2;
                } else {
                    i += 1;
                }
                start = i;
            }
            b'\n' => {
                out.push(text[start..i].to_string());
                i += 1;
                start = i;
            }
            _ => {
                i += 1;
            }
        }
    }
    out.push(text[start..].to_string());
    out
}

/// Join lines with `eol`, re-appending one terminator when the original had one.
pub fn join(lines: &[String], eol: &str, trailing_eol: bool) -> String {
    let joined = lines.join(eol);
    if trailing_eol && !joined.is_empty() {
        format!("{joined}{eol}")
    } else {
        joined
    }
}
