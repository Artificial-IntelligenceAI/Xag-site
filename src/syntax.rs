//! A tokeniser for Xag, for showing code on a page.
//!
//! It exists to colour, not to understand: nothing here knows what a program
//! means, and nothing here is a second implementation of the language. The
//! compiler at `Xag-lang` remains the only authority on that.
//!
//! What makes it short is the language rather than any cleverness here. A
//! quoted thing is a name wherever it is met and a starred thing is a written
//! value wherever it is met, so neither one has to be read twice or read in the
//! light of what came before it. There is no lookahead below, and no state
//! carried between tokens — position is never consulted, because Xag never asks
//! it to be.

/// What a run of characters is. There are two marks in Xag, so there are two
/// marked kinds here, and there will only ever be two.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// `'greeting'` — a name, everywhere and always.
    Name,
    /// `*hello*` — a written value, everywhere and always.
    Value,
    /// `\n` — an escape, which stands beside the text rather than inside it.
    Escape,
    /// `# ...` to the end of the line.
    Comment,
    /// A bare word: a function, a type, or a segment of a chain.
    Word,
    /// Brackets, dots, semicolons, operators.
    Punct,
    /// Spaces and newlines, kept so the code lays out as it was written.
    Space,
}

impl Kind {
    /// The CSS class this kind is drawn with.
    pub fn class(self) -> &'static str {
        match self {
            Kind::Name => "tk-name",
            Kind::Value => "tk-value",
            Kind::Escape => "tk-escape",
            Kind::Comment => "tk-comment",
            Kind::Word => "tk-word",
            Kind::Punct => "tk-punct",
            Kind::Space => "tk-space",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
}

/// A word is letters, digits and `_`, joined by `-`.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Reads a marked run: `'…'` or `*…*`. Inside a mark, a backslash writes the
/// closing mark itself — the one character that could not otherwise appear
/// there — so it is the only thing that has to be stepped over.
fn take_marked(chars: &[char], start: usize, mark: char) -> (String, usize) {
    let mut text = String::from(mark);
    let mut i = start + 1;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() && chars[i + 1] == mark {
            text.push('\\');
            text.push(mark);
            i += 2;
            continue;
        }
        text.push(c);
        i += 1;
        if c == mark {
            break;
        }
    }
    (text, i)
}

/// Splits Xag source into coloured runs.
///
/// Every character of the input appears in exactly one token, in order, so the
/// tokens join back into the source they came from. Nothing is dropped and
/// nothing is invented — see `rejoins_exactly` below.
pub fn tokenize(src: &str) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];

        // A comment runs to the end of its line.
        if c == '#' {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            out.push(Token { kind: Kind::Comment, text: chars[start..i].iter().collect() });
            continue;
        }

        // The two marks.
        if c == '\'' || c == '*' {
            let (text, next) = take_marked(&chars, i, c);
            let kind = if c == '\'' { Kind::Name } else { Kind::Value };
            out.push(Token { kind, text });
            i = next;
            continue;
        }

        // An escape stands outside the text, as its own item.
        if c == '\\' && i + 1 < chars.len() && matches!(chars[i + 1], 'n' | 't' | 'r' | '\\') {
            out.push(Token { kind: Kind::Escape, text: chars[i..i + 2].iter().collect() });
            i += 2;
            continue;
        }

        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            out.push(Token { kind: Kind::Space, text: chars[start..i].iter().collect() });
            continue;
        }

        // A bare word. `-` joins it only between two word characters, which is
        // what keeps `sum-to` one word and `count - *1*` three tokens.
        if is_word_char(c) {
            let start = i;
            while i < chars.len() {
                if is_word_char(chars[i]) {
                    i += 1;
                } else if chars[i] == '-'
                    && i + 1 < chars.len()
                    && is_word_char(chars[i + 1])
                    && i > start
                {
                    i += 1;
                } else {
                    break;
                }
            }
            out.push(Token { kind: Kind::Word, text: chars[start..i].iter().collect() });
            continue;
        }

        out.push(Token { kind: Kind::Punct, text: c.to_string() });
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(Kind, String)> {
        tokenize(src)
            .into_iter()
            .filter(|t| t.kind != Kind::Space)
            .map(|t| (t.kind, t.text))
            .collect()
    }

    /// The property the page depends on: colouring never loses the program.
    #[test]
    fn rejoins_exactly() {
        for src in [
            include_str!("../samples/counting.xag"),
            include_str!("../samples/grouping.xag"),
            include_str!("../samples/borrowing.xag"),
            "var.str 'it\\'s' = [*a\\*b*]; # end",
        ] {
            let joined: String = tokenize(src).into_iter().map(|t| t.text).collect();
            assert_eq!(joined, src);
        }
    }

    #[test]
    fn a_quoted_thing_is_a_name_and_a_starred_thing_is_a_value() {
        assert_eq!(
            kinds("var.str 'a' = [*b*];"),
            vec![
                (Kind::Word, "var".into()),
                (Kind::Punct, ".".into()),
                (Kind::Word, "str".into()),
                (Kind::Name, "'a'".into()),
                (Kind::Punct, "=".into()),
                (Kind::Punct, "[".into()),
                (Kind::Value, "*b*".into()),
                (Kind::Punct, "]".into()),
                (Kind::Punct, ";".into()),
            ]
        );
    }

    /// `*1000*` is a number under `int64` and four characters under `str`, and
    /// this tokeniser does not care which — it is a written value either way.
    #[test]
    fn a_value_is_a_value_whatever_its_type_says() {
        assert_eq!(kinds("*1000*")[0].0, Kind::Value);
        assert_eq!(kinds("str:*1000*")[2].0, Kind::Value);
    }

    #[test]
    fn a_mark_may_be_written_inside_its_own_mark() {
        assert_eq!(kinds("'it\\'s'"), vec![(Kind::Name, "'it\\'s'".into())]);
        assert_eq!(kinds("*a\\*b*"), vec![(Kind::Value, "*a\\*b*".into())]);
    }

    /// `-` joins a word between two word characters, and is subtraction
    /// everywhere else.
    #[test]
    fn a_dash_joins_a_word_but_does_not_join_a_subtraction() {
        assert_eq!(kinds("sum-to"), vec![(Kind::Word, "sum-to".into())]);
        assert_eq!(
            kinds("count - *1*"),
            vec![
                (Kind::Word, "count".into()),
                (Kind::Punct, "-".into()),
                (Kind::Value, "*1*".into()),
            ]
        );
    }

    /// `*a\nb*` is a backslash and an `n`, because escapes stand outside.
    #[test]
    fn an_escape_stands_outside_the_text() {
        assert_eq!(
            kinds("[*line* \\n *two*]"),
            vec![
                (Kind::Punct, "[".into()),
                (Kind::Value, "*line*".into()),
                (Kind::Escape, "\\n".into()),
                (Kind::Value, "*two*".into()),
                (Kind::Punct, "]".into()),
            ]
        );
        assert_eq!(kinds("*a\\nb*"), vec![(Kind::Value, "*a\\nb*".into())]);
    }

    /// A declaration marks the name it gives, so a function's name is a name
    /// where it is declared and a bare word where it is called. The tokeniser
    /// needed nothing added for this — it already coloured what it was shown,
    /// which is the point of there being only two marks.
    #[test]
    fn a_declared_name_is_a_name_even_when_it_names_a_function() {
        assert_eq!(
            kinds("fn.int64 'sum-to' [int64 'n']"),
            vec![
                (Kind::Word, "fn".into()),
                (Kind::Punct, ".".into()),
                (Kind::Word, "int64".into()),
                (Kind::Name, "'sum-to'".into()),
                (Kind::Punct, "[".into()),
                (Kind::Word, "int64".into()),
                (Kind::Name, "'n'".into()),
                (Kind::Punct, "]".into()),
            ]
        );

        // And calling it is a bare word, as it always was.
        assert_eq!(kinds("sum-to['LIMIT']")[0], (Kind::Word, "sum-to".into()));
        assert_eq!(kinds("struct 'point' [int64 'x']")[1].0, Kind::Name);
    }

    #[test]
    fn a_name_may_hold_anything_at_all() {
        assert_eq!(
            kinds("var.str 'a name with spaces \u{1f642}'")[3].0,
            Kind::Name
        );
    }

    #[test]
    fn a_comment_runs_to_the_end_of_its_line() {
        assert_eq!(
            kinds("# 'not' a *name*\nvar"),
            vec![
                (Kind::Comment, "# 'not' a *name*".into()),
                (Kind::Word, "var".into()),
            ]
        );
    }
}
