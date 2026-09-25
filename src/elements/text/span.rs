use vello::peniko::Color;

/// Explicit inline span: own text plus style flags. Spans concatenate
/// in order inside `FormattedText`; adjacent spans with identical
/// style merge during markdown parsing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    /// Monospace family.
    pub code: bool,
    /// Fixed text color (`None` inherits the base foreground).
    pub color: Option<Color>,
    /// Underline on/off.
    pub underline: bool,
    /// Underline color (`None` inherits the text color).
    pub underline_color: Option<Color>,
    /// Strikethrough on/off.
    pub strikethrough: bool,
    /// Strikethrough color (`None` inherits the text color).
    pub strikethrough_color: Option<Color>,
    /// Link target. Renders in the accent color, underlined, and
    /// fires `on_link` on click.
    pub link: Option<String>,
}

impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn underline_color(mut self, color: Color) -> Self {
        self.underline = true;
        self.underline_color = Some(color);
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn strikethrough_color(mut self, color: Color) -> Self {
        self.strikethrough = true;
        self.strikethrough_color = Some(color);
        self
    }

    pub fn link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self
    }

    fn style_key(&self) -> (bool, bool, bool, Option<Color>, bool, Option<Color>, bool, Option<Color>, Option<&str>) {
        (
            self.bold,
            self.italic,
            self.code,
            self.color,
            self.underline,
            self.underline_color,
            self.strikethrough,
            self.strikethrough_color,
            self.link.as_deref(),
        )
    }

    /// Same style (ignoring text)? Used to merge adjacent spans.
    fn same_style(&self, other: &Self) -> bool {
        self.style_key() == other.style_key()
    }
}

#[derive(Clone, Copy, Default)]
struct Flags {
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
}

/// Parse inline markdown into spans. Supported subset (CommonMark-like,
/// single paragraph):
///
/// - `**bold**`, `*italic*`, `_italic_`, `***bold italic***`
/// - `~~strikethrough~~`
/// - `` `code` `` (monospace, literal inside)
/// - `[label](url)` (link; the label parses inline formatting)
/// - `\` escapes the next character
///
/// Markers need a matching closer on the same input; unmatched markers
/// stay literal. Single `*`/`_` openers must not be followed by
/// whitespace and closers must not be preceded by whitespace, so
/// `foo_bar` keeps its underscores.
pub fn parse_markdown(source: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    parse_into(source, Flags::default(), None, &mut spans);
    merge_spans(&mut spans);
    spans
}

fn push_text(out: &mut Vec<Span>, text: &str, flags: Flags, link: Option<&str>) {
    if text.is_empty() {
        return;
    }
    out.push(Span {
        text: text.to_string(),
        bold: flags.bold,
        italic: flags.italic,
        code: flags.code,
        strikethrough: flags.strike,
        link: link.map(str::to_string),
        ..Span::default()
    });
}

fn merge_spans(spans: &mut Vec<Span>) {
    let mut merged: Vec<Span> = Vec::with_capacity(spans.len());
    for span in spans.drain(..) {
        if let Some(last) = merged.last_mut() {
            if last.same_style(&span) {
                last.text.push_str(&span.text);
                continue;
            }
        }
        merged.push(span);
    }
    *spans = merged;
}

/// Byte offset of the closing `marker` in `hay`, or `None`.
/// A lone `*` closer must not touch another `*` or whitespace; a `_`
/// closer needs an alphanumeric before it and no alphanumeric after
/// it (flanking), so `foo_bar_baz` never splits.
fn find_closer(hay: &str, marker: &str) -> Option<usize> {
    let bytes = hay.as_bytes();
    let m = marker.len();
    let mut i = 0;
    while i + m <= bytes.len() {
        if &hay[i..i + m] == marker {
            let ok = match marker {
                "*" => {
                    let prev = if i == 0 { b' ' } else { bytes[i - 1] };
                    let lone = (i == 0 || bytes[i - 1] != b'*')
                        && (i + 1 >= bytes.len() || bytes[i + 1] != b'*');
                    lone && !prev.is_ascii_whitespace()
                }
                "_" => {
                    let prev_alpha = hay[..i]
                        .chars()
                        .next_back()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);
                    let next_alpha = hay[i + 1..]
                        .chars()
                        .next()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);
                    prev_alpha && !next_alpha
                }
                _ => true,
            };
            if ok {
                return Some(i);
            }
        }
        // Advance one char to stay on UTF-8 boundaries.
        i += hay[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    None
}

/// Can `marker` open at the start of `hay` (`prev` is the char before
/// it, if any)? Single `*` must not be followed by whitespace; `_`
/// additionally needs a non-alphanumeric before it (flanking).
fn opener_ok(hay: &str, marker: &str, prev: Option<char>) -> bool {
    match marker {
        "*" => hay
            .chars()
            .next()
            .map(|c| !c.is_whitespace())
            .unwrap_or(false),
        "_" => {
            let prev_ok = prev.map(|c| !c.is_alphanumeric()).unwrap_or(true);
            let next_ok = hay
                .chars()
                .next()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
            prev_ok && next_ok
        }
        _ => true,
    }
}

/// Byte offset of `]` closing a link label (no nesting), or `None`.
fn find_label_end(hay: &str) -> Option<usize> {
    let mut i = 0;
    while i < hay.len() {
        match hay.as_bytes()[i] {
            b']' => return Some(i),
            b'\\' => i += 1,
            _ => {}
        }
        i += hay[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    None
}

fn parse_into(s: &str, flags: Flags, link: Option<&str>, out: &mut Vec<Span>) {
    let mut i = 0;
    let mut plain_start = 0;
    // Flush pending plain text without a closure (which would borrow
    // `plain_start` across its assignments).
    macro_rules! flush {
        ($end:expr) => {
            push_text(out, &s[plain_start..$end], flags, link)
        };
    }
    while i < s.len() {
        let rest = &s[i..];
        // Escape: backslash drops, next char stays literal.
        if rest.starts_with('\\') && rest.len() > 1 {
            let mut chars = rest.char_indices();
            chars.next();
            if let Some((off, _)) = chars.next() {
                let end = &rest[off..];
                let skip = end.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                // Keep the backslash out: flush before it, skip it.
                flush!(i);
                i += 1;
                plain_start = i;
                i += skip;
                continue;
            }
        }
        if flags.code {
            if rest.starts_with('`') {
                flush!(i);
                let mut next = flags;
                next.code = false;
                i += 1;
                // Continue after the code span with code off. The text
                // after was already flushed; recurse for the remainder.
                let mut tail = Vec::new();
                parse_into(&s[i..], next, link, &mut tail);
                out.append(&mut tail);
                return;
            }
            i += rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            continue;
        }
        // Longest markers first.
        if rest.starts_with("***") {
            if opener_ok(&rest[3..], "***", None) {
                if let Some(end) = find_closer(&rest[3..], "***") {
                    flush!(i);
                    let mut inner = flags;
                    inner.bold = true;
                    inner.italic = true;
                    parse_into(&rest[3..3 + end], inner, link, out);
                    i += 3 + end + 3;
                    plain_start = i;
                    continue;
                }
            }
        } else if rest.starts_with("**") {
            if opener_ok(&rest[2..], "**", None) {
                if let Some(end) = find_closer(&rest[2..], "**") {
                    flush!(i);
                    let mut inner = flags;
                    inner.bold = true;
                    parse_into(&rest[2..2 + end], inner, link, out);
                    i += 2 + end + 2;
                    plain_start = i;
                    continue;
                }
            }
        } else if rest.starts_with("~~") {
            if opener_ok(&rest[2..], "~~", None) {
                if let Some(end) = find_closer(&rest[2..], "~~") {
                    flush!(i);
                    let mut inner = flags;
                    inner.strike = true;
                    parse_into(&rest[2..2 + end], inner, link, out);
                    i += 2 + end + 2;
                    plain_start = i;
                    continue;
                }
            }
        } else if rest.starts_with('*') {
            if opener_ok(&rest[1..], "*", None) {
                if let Some(end) = find_closer(&rest[1..], "*") {
                    flush!(i);
                    let mut inner = flags;
                    inner.italic = true;
                    parse_into(&rest[1..1 + end], inner, link, out);
                    i += 1 + end + 1;
                    plain_start = i;
                    continue;
                }
            }
        } else if rest.starts_with('_') {
            let prev = s[..i].chars().next_back();
            if opener_ok(&rest[1..], "_", prev) {
                if let Some(end) = find_closer(&rest[1..], "_") {
                    flush!(i);
                    let mut inner = flags;
                    inner.italic = true;
                    parse_into(&rest[1..1 + end], inner, link, out);
                    i += 1 + end + 1;
                    plain_start = i;
                    continue;
                }
            }
        } else if rest.starts_with('`') {
            if let Some(end) = find_closer(&rest[1..], "`") {
                flush!(i);
                let mut inner = flags;
                inner.code = true;
                push_text(out, &rest[1..1 + end], inner, link);
                i += 1 + end + 1;
                plain_start = i;
                continue;
            }
        } else if rest.starts_with('[') {
            if let Some(label_end) = find_label_end(&rest[1..]) {
                let after = &rest[1 + label_end + 1..];
                if after.starts_with('(') {
                    if let Some(url_end) = after.find(')') {
                        flush!(i);
                        let label = &rest[1..1 + label_end];
                        let url = after[1..url_end].to_string();
                        parse_into(label, flags, Some(&url), out);
                        i += 1 + label_end + 1 + url_end + 1;
                        plain_start = i;
                        continue;
                    }
                }
            }
        }
        i += rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    flush!(s.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_stays_single_span() {
        let spans = parse_markdown("Hello, SwiftUI!");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "Hello, SwiftUI!");
        assert!(!spans[0].bold);
    }

    #[test]
    fn bold_italic_strike_code() {
        let spans = parse_markdown("**bold** *italic* ~~strike~~ `code`");
        let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
        assert!(texts.contains(&"bold"));
        assert!(texts.contains(&"italic"));
        assert!(texts.contains(&"strike"));
        assert!(texts.contains(&"code"));
        assert!(spans.iter().find(|s| s.text == "bold").unwrap().bold);
        assert!(spans.iter().find(|s| s.text == "italic").unwrap().italic);
        assert!(spans.iter().find(|s| s.text == "strike").unwrap().strikethrough);
        assert!(spans.iter().find(|s| s.text == "code").unwrap().code);
    }

    #[test]
    fn triple_sets_bold_and_italic() {
        let spans = parse_markdown("***both***");
        assert_eq!(spans.len(), 1);
        assert!(spans[0].bold && spans[0].italic);
    }

    #[test]
    fn nested_bold_italic() {
        let spans = parse_markdown("**Bold and *italic* together**");
        assert!(spans.iter().all(|s| s.bold));
        assert!(spans.iter().find(|s| s.text == "italic").unwrap().italic);
    }

    #[test]
    fn link_parses_label_and_url() {
        let spans = parse_markdown("tap [here](https://example.com/x) now");
        let link = spans.iter().find(|s| s.text == "here").unwrap();
        assert_eq!(link.link.as_deref(), Some("https://example.com/x"));
    }

    #[test]
    fn unmatched_markers_stay_literal() {
        let spans = parse_markdown("a ** b * c");
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "a ** b * c");
    }

    #[test]
    fn underscore_needs_word() {
        let spans = parse_markdown("foo_bar_baz");
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "foo_bar_baz");
        assert!(spans.iter().all(|s| !s.italic));
    }

    #[test]
    fn escape_drops_backslash() {
        let spans = parse_markdown("a \\* b");
        let joined: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "a * b");
        assert!(spans.iter().all(|s| !s.italic));
    }
}

