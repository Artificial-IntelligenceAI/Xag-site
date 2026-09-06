//! A very small markup, so the site's prose can live in one place and be
//! rendered twice.
//!
//! The page is drawn two ways: as HTML written at build time, for anything that
//! will not run WebAssembly, and by the app once it has. Both read the same
//! strings through here, which is the only reason the two cannot drift apart.
//!
//! What it understands, and nothing else:
//!
//! ```text
//! `code`            a literal
//! `code`{name}      a literal coloured as a name, `code`{value} as a value
//! **strong**        emphasis that matters
//! _em_              emphasis that does not
//! [words](where)    a link
//! ```
//!
//! Emphasis is `_` rather than `*` on purpose: `*` is one of Xag's two marks
//! and appears constantly in the prose. A markup that fought the language it
//! describes would be a poor one.

/// The five characters that mean something else in HTML.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Turns a run of prose into HTML.
///
/// Anything it does not recognise is text, escaped. There is no way to write
/// raw HTML through it, which is what keeps a stray angle bracket in a sentence
/// about generics from becoming a tag.
pub fn to_html(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len() + 32);
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '`' => {
                if let Some(end) = find(&chars, i + 1, '`') {
                    let body: String = chars[i + 1..end].iter().collect();
                    i = end + 1;
                    // An optional {name} or {value} colours it.
                    let class = if i < chars.len() && chars[i] == '{' {
                        find(&chars, i + 1, '}').map(|close| {
                            let kind: String = chars[i + 1..close].iter().collect();
                            i = close + 1;
                            kind
                        })
                    } else {
                        None
                    };
                    match class.as_deref() {
                        Some(kind @ ("name" | "value")) => {
                            out.push_str(&format!(
                                "<code class=\"tk-{kind}\">{}</code>",
                                escape(&body)
                            ));
                        }
                        _ => out.push_str(&format!("<code>{}</code>", escape(&body))),
                    }
                    continue;
                }
            }
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                if let Some(end) = find_pair(&chars, i + 2) {
                    let body: String = chars[i + 2..end].iter().collect();
                    out.push_str(&format!("<strong>{}</strong>", to_html(&body)));
                    i = end + 2;
                    continue;
                }
            }
            '_' => {
                if let Some(end) = find(&chars, i + 1, '_') {
                    let body: String = chars[i + 1..end].iter().collect();
                    out.push_str(&format!("<em>{}</em>", to_html(&body)));
                    i = end + 1;
                    continue;
                }
            }
            '[' => {
                if let Some(close) = find(&chars, i + 1, ']') {
                    if close + 1 < chars.len() && chars[close + 1] == '(' {
                        if let Some(end) = find(&chars, close + 2, ')') {
                            let text: String = chars[i + 1..close].iter().collect();
                            let href: String = chars[close + 2..end].iter().collect();
                            out.push_str(&format!(
                                "<a href=\"{}\">{}</a>",
                                escape(&href),
                                to_html(&text)
                            ));
                            i = end + 1;
                            continue;
                        }
                    }
                }
            }
            _ => {}
        }

        // Anything unmatched is just itself.
        out.push_str(&escape(&chars[i].to_string()));
        i += 1;
    }

    out
}

fn find(chars: &[char], from: usize, needle: char) -> Option<usize> {
    (from..chars.len()).find(|&i| chars[i] == needle)
}

fn find_pair(chars: &[char], from: usize) -> Option<usize> {
    (from..chars.len().saturating_sub(1)).find(|&i| chars[i] == '*' && chars[i + 1] == '*')
}

/// The prose with every mark taken out, for a meta description or a title.
pub fn to_text(source: &str) -> String {
    let html = to_html(source);
    let mut out = String::with_capacity(html.len());
    let mut inside = false;
    for c in html.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words_come_through_as_they_went_in() {
        assert_eq!(to_html("just a sentence."), "just a sentence.");
    }

    /// The whole reason this exists rather than an HTML string: a stray angle
    /// bracket in a sentence must not become a tag.
    #[test]
    fn html_in_the_prose_is_text() {
        assert_eq!(
            to_html("a <script> and an & ampersand"),
            "a &lt;script&gt; and an &amp; ampersand"
        );
    }

    #[test]
    fn the_four_marks() {
        assert_eq!(to_html("`int64`"), "<code>int64</code>");
        assert_eq!(to_html("**yes**"), "<strong>yes</strong>");
        assert_eq!(to_html("_quietly_"), "<em>quietly</em>");
        assert_eq!(
            to_html("[the repo](https://example.org)"),
            "<a href=\"https://example.org\">the repo</a>"
        );
    }

    /// Xag's own marks appear constantly in the prose and must survive it.
    #[test]
    fn xags_marks_are_not_markup() {
        assert_eq!(
            to_html("`*1000*` is a number under `int64`"),
            "<code>*1000*</code> is a number under <code>int64</code>"
        );
        assert_eq!(to_html("`'name'`"), "<code>&#39;name&#39;</code>");
    }

    #[test]
    fn a_literal_may_be_coloured() {
        assert_eq!(
            to_html("`*…*`{value}"),
            "<code class=\"tk-value\">*…*</code>"
        );
        assert_eq!(to_html("`'…'`{name}"), "<code class=\"tk-name\">&#39;…&#39;</code>");
    }

    /// An unclosed mark is a typo, not a reason to lose the sentence.
    #[test]
    fn an_unfinished_mark_is_left_alone() {
        assert_eq!(to_html("a ` that never closes"), "a ` that never closes");
        assert_eq!(to_html("**never mind"), "**never mind");
    }

    #[test]
    fn text_strips_everything() {
        assert_eq!(
            to_text("**Xag** is `int64` and [a link](https://example.org)"),
            "Xag is int64 and a link"
        );
    }
}
