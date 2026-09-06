//! A very small interpreter, for feeling out the syntax without installing
//! anything.
//!
//! **It is not the compiler.** It is a few hundred lines written for this
//! website, it knows one type and a handful of statements, and it checks none
//! of the things Xag is actually careful about — no ownership, no borrowing, no
//! moves. Where it disagrees with `xagc`, `xagc` is right and this is wrong.
//!
//! It exists because trying a syntax is a different question from trusting a
//! language, and the first one should not cost a checkout and a build of LLVM.
//!
//! What keeps it honest is that everything it can run is run against the real
//! compiler too — see `src/verify_play.rs`, which puts every example through
//! both and complains if they differ. That is a thing somebody has to run; it
//! is not proof against the language moving underneath.

use crate::syntax::{tokenize, Kind};

/// What the playground understands. Deliberately short, and worth reading
/// before the first surprise.
pub const SUPPORTED: [&str; 6] = [
    "`var` and `var.mut` inside `START`, and `const` above it, of type `int64`",
    "`set`, on a name whose chain said `mut`",
    "arithmetic: `+`, `-`, `x`, `/` and `mod`, with mathematics' own precedence",
    "`print.stdout`, with `str:` text, names, sums and `\\n`",
    "`loop.range.int64`, over a first and last value",
    "one `START` block, and comments",
];

/// What it does not, which is the longer and more important list.
pub const UNSUPPORTED: [&str; 6] = [
    "**ownership** — nothing is moved, borrowed or checked. This is the half of Xag that matters most and the playground has none of it",
    "every type but `int64`: no `str` variables, no `bin`, no `deci`, no `bool`",
    "functions, `struct`, `many`, `if`, `when`, `give`, `read.stdin`",
    "the compiler's diagnostics — errors here are this interpreter's own words, and much poorer",
    "anything about how fast a real Xag program runs",
    "being right. Where it and `xagc` disagree, `xagc` is right",
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Op {
    fn word(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "x",
            Op::Div => "/",
            Op::Mod => "mod",
        }
    }

    /// `x` and `/` before `+` and `-`, as everybody learned before they met a
    /// keyboard. `mod` has no agreed place and is handled separately.
    fn tight(self) -> bool {
        matches!(self, Op::Mul | Op::Div)
    }
}

enum Expr {
    Num(i64),
    Name(String),
    Bin(Box<Expr>, Op, Box<Expr>),
}

enum Item {
    Text(String),
    Newline,
    Value(Expr),
}

enum Stmt {
    Declare { name: String, mutable: bool, value: Expr },
    Set { name: String, value: Expr },
    Print(Vec<Item>),
    Loop { name: String, from: Expr, to: Expr, body: Vec<Stmt> },
}

struct Tok {
    kind: Kind,
    text: String,
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

type Fallible<T> = Result<T, String>;

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn is(&self, kind: Kind, text: &str) -> bool {
        self.peek().is_some_and(|t| t.kind == kind && t.text == text)
    }

    fn take(&mut self) -> Fallible<&Tok> {
        let tok = self.toks.get(self.at).ok_or("the program stops in the middle of something")?;
        self.at += 1;
        Ok(tok)
    }

    fn want(&mut self, kind: Kind, text: &str) -> Fallible<()> {
        if self.is(kind, text) {
            self.at += 1;
            Ok(())
        } else {
            let found = self.peek().map_or("the end".into(), |t| format!("`{}`", t.text));
            Err(format!("expected `{text}`, found {found}"))
        }
    }

    fn word(&mut self) -> Fallible<String> {
        let tok = self.take()?;
        if tok.kind == Kind::Word {
            Ok(tok.text.clone())
        } else {
            Err(format!("expected a word, found `{}`", tok.text))
        }
    }

    fn name(&mut self) -> Fallible<String> {
        let tok = self.take()?;
        if tok.kind == Kind::Name {
            Ok(tok.text.trim_matches('\'').to_string())
        } else {
            Err(format!("expected a name in quotes, found `{}`", tok.text))
        }
    }

    /// The chain after `var`, which the playground reads only far enough to
    /// find `mut` and the type.
    fn chain(&mut self) -> Fallible<bool> {
        let mut mutable = false;
        let mut kind = None;
        while self.is(Kind::Punct, ".") {
            self.at += 1;
            let word = self.word()?;
            match word.as_str() {
                "mut" => mutable = true,
                "int64" => kind = Some(word),
                other => {
                    return Err(format!(
                        "the playground only knows `int64`, and `mut`. It does not know `{other}`"
                    ))
                }
            }
        }
        if kind.is_none() {
            return Err("a declaration needs a type, and here that means `int64`".into());
        }
        Ok(mutable)
    }

    fn block(&mut self) -> Fallible<Vec<Stmt>> {
        self.want(Kind::Punct, "{")?;
        let mut out = Vec::new();
        while !self.is(Kind::Punct, "}") {
            if self.peek().is_none() {
                return Err("a `{` is never closed".into());
            }
            out.push(self.statement()?);
        }
        self.at += 1;
        Ok(out)
    }

    fn statement(&mut self) -> Fallible<Stmt> {
        let word = self.word()?;
        match word.as_str() {
            "const" => Err(
                "`const` is not something a `var` chain says. A `const` is declared \
                 outside `START`, above it."
                    .into(),
            ),
            "var" => {
                let mutable = self.chain()?;
                let name = self.name()?;
                self.want(Kind::Punct, "=")?;
                self.want(Kind::Punct, "[")?;
                let value = self.expr()?;
                self.want(Kind::Punct, "]")?;
                self.want(Kind::Punct, ";")?;
                Ok(Stmt::Declare { name, mutable, value })
            }
            "set" => {
                let name = self.name()?;
                self.want(Kind::Punct, "=")?;
                self.want(Kind::Punct, "[")?;
                let value = self.expr()?;
                self.want(Kind::Punct, "]")?;
                self.want(Kind::Punct, ";")?;
                Ok(Stmt::Set { name, value })
            }
            "print" => {
                self.want(Kind::Punct, ".")?;
                let where_to = self.word()?;
                if where_to != "stdout" {
                    return Err(format!("the playground can only print to `stdout`, not `{where_to}`"));
                }
                self.want(Kind::Punct, "[")?;
                let mut items = Vec::new();
                while !self.is(Kind::Punct, "]") {
                    items.push(self.item()?);
                }
                self.at += 1;
                self.want(Kind::Punct, ";")?;
                Ok(Stmt::Print(items))
            }
            "loop" => {
                self.want(Kind::Punct, ".")?;
                let sort = self.word()?;
                if sort != "range" {
                    return Err(format!("the playground only knows `loop.range`, not `loop.{sort}`"));
                }
                self.chain()?;
                let name = self.name()?;
                self.want(Kind::Punct, "=")?;
                self.want(Kind::Punct, "[")?;
                let from = self.expr()?;
                self.want(Kind::Punct, ",")?;
                let to = self.expr()?;
                self.want(Kind::Punct, "]")?;
                let body = self.block()?;
                Ok(Stmt::Loop { name, from, to, body })
            }
            other => Err(format!(
                "the playground does not know `{other}`. It knows `var`, `const`, `set`, `print` and `loop`"
            )),
        }
    }

    fn item(&mut self) -> Fallible<Item> {
        // `str:*…*` — text, with its type said because a print list has nothing
        // else to say it.
        if self.is(Kind::Word, "str") {
            self.at += 1;
            self.want(Kind::Punct, ":")?;
            let tok = self.take()?;
            if tok.kind != Kind::Value {
                return Err(format!("`str:` wants written text, found `{}`", tok.text));
            }
            return Ok(Item::Text(unwritten(&tok.text)));
        }
        // `int64:` says the type of what follows, which may be a whole sum and
        // not only one written number.
        if self.is(Kind::Word, "int64") {
            self.at += 1;
            self.want(Kind::Punct, ":")?;
            return Ok(Item::Value(self.expr()?));
        }
        if self.peek().is_some_and(|t| t.kind == Kind::Escape) {
            let tok = self.take()?;
            return match tok.text.as_str() {
                "\\n" => Ok(Item::Newline),
                "\\t" => Ok(Item::Text("\t".into())),
                other => Err(format!("the playground does not know the escape `{other}`")),
            };
        }
        if self.peek().is_some_and(|t| t.kind == Kind::Value) {
            let tok = self.take()?;
            return Err(format!(
                "`{}` has no type here. A print list says nothing about what is in it, so a written value has to: `str:{}` or `int64:{}`",
                tok.text, tok.text, tok.text
            ));
        }
        Ok(Item::Value(self.expr()?))
    }

    /// Everything at one level, then folded by precedence — which is the only
    /// way to notice that `mod` was mixed with something it has no agreed order
    /// against.
    fn expr(&mut self) -> Fallible<Expr> {
        let mut operands = vec![self.primary()?];
        let mut ops: Vec<Op> = Vec::new();

        loop {
            let op = match self.peek() {
                Some(t) if t.kind == Kind::Punct && t.text == "+" => Op::Add,
                Some(t) if t.kind == Kind::Punct && t.text == "-" => Op::Sub,
                Some(t) if t.kind == Kind::Word && t.text == "x" => Op::Mul,
                Some(t) if t.kind == Kind::Punct && t.text == "/" => Op::Div,
                Some(t) if t.kind == Kind::Word && t.text == "mod" => Op::Mod,
                _ => break,
            };
            self.at += 1;
            ops.push(op);
            operands.push(self.primary()?);
        }

        // `mod` against anything else, or against itself, has no order anybody
        // agreed on — so it is bracketed or it is an error naming both
        // readings, which is what the compiler does too.
        if ops.contains(&Op::Mod) && ops.len() > 1 {
            let other = ops.iter().find(|o| **o != Op::Mod).copied();
            return Err(match other {
                Some(other) => format!(
                    "`mod` and `{}` have no agreed order, so this could be read two ways. Brackets say which.",
                    other.word()
                ),
                None => "`mod` is not associative, so `a mod b mod c` could be read two ways. Brackets say which.".into(),
            });
        }

        Ok(fold(operands, ops))
    }

    fn primary(&mut self) -> Fallible<Expr> {
        if self.is(Kind::Punct, "(") {
            self.at += 1;
            let inner = self.expr()?;
            self.want(Kind::Punct, ")")?;
            return Ok(inner);
        }
        let tok = self.take()?;
        match tok.kind {
            Kind::Value => {
                let text = unwritten(&tok.text);
                text.parse::<i64>()
                    .map(Expr::Num)
                    .map_err(|_| format!("`{text}` is not a whole number the playground can read"))
            }
            Kind::Name => Ok(Expr::Name(tok.text.trim_matches('\'').to_string())),
            _ => Err(format!("expected a value or a name, found `{}`", tok.text)),
        }
    }
}

/// Folds a flat run into a tree, tighter operators first.
fn fold(mut operands: Vec<Expr>, mut ops: Vec<Op>) -> Expr {
    let mut i = 0;
    while i < ops.len() {
        if ops[i].tight() {
            let right = operands.remove(i + 1);
            let left = operands.remove(i);
            operands.insert(i, Expr::Bin(Box::new(left), ops.remove(i), Box::new(right)));
        } else {
            i += 1;
        }
    }
    while !ops.is_empty() {
        let right = operands.remove(1);
        let left = operands.remove(0);
        operands.insert(0, Expr::Bin(Box::new(left), ops.remove(0), Box::new(right)));
    }
    operands.pop().expect("an expression has at least one operand")
}

/// The text inside a pair of marks, with the escapes a mark may hold.
fn unwritten(marked: &str) -> String {
    let inner = marked.trim_matches('*');
    inner.replace("\\*", "*")
}

struct Bound {
    value: i64,
    mutable: bool,
}

/// Runs a program, and gives back what it printed.
pub fn run(source: &str) -> Result<String, String> {
    let toks: Vec<Tok> = tokenize(source)
        .into_iter()
        .filter(|t| !matches!(t.kind, Kind::Space | Kind::Comment))
        .map(|t| Tok { kind: t.kind, text: t.text })
        .collect();

    let mut parser = Parser { toks, at: 0 };

    // Whatever stands above `START`, which for the playground is `const` and
    // nothing else.
    let mut top: Vec<Stmt> = Vec::new();
    loop {
        let word = parser.word()?;
        match word.as_str() {
            "START" => break,
            "const" => {
                parser.chain()?;
                let name = parser.name()?;
                parser.want(Kind::Punct, "=")?;
                parser.want(Kind::Punct, "[")?;
                let value = parser.expr()?;
                parser.want(Kind::Punct, "]")?;
                parser.want(Kind::Punct, ";")?;
                top.push(Stmt::Declare { name, mutable: false, value });
            }
            other => {
                return Err(format!(
                    "above `START` the playground knows only `const`, and found `{other}`"
                ))
            }
        }
    }
    let body = parser.block()?;
    if parser.peek().is_some() {
        return Err("there is something after the `START` block, which the playground cannot run".into());
    }

    let mut names: Vec<(String, Bound)> = Vec::new();
    let mut out = String::new();
    let mut steps = 0;
    perform(&top, &mut names, &mut out, &mut steps)?;
    perform(&body, &mut names, &mut out, &mut steps)?;
    Ok(out)
}

/// A budget, so a loop that never ends stops being the reader's problem.
const MOST_STEPS: u64 = 2_000_000;

fn perform(
    body: &[Stmt],
    names: &mut Vec<(String, Bound)>,
    out: &mut String,
    steps: &mut u64,
) -> Result<(), String> {
    for stmt in body {
        *steps += 1;
        if *steps > MOST_STEPS {
            return Err("the playground stopped: that is more work than it will do in a browser".into());
        }
        match stmt {
            Stmt::Declare { name, mutable, value } => {
                let value = eval(value, names)?;
                if names.iter().any(|(n, _)| n == name) {
                    return Err(format!("`'{name}'` is declared twice"));
                }
                names.push((name.clone(), Bound { value, mutable: *mutable }));
            }
            Stmt::Set { name, value } => {
                let value = eval(value, names)?;
                let bound = names
                    .iter_mut()
                    .find(|(n, _)| n == name)
                    .map(|(_, b)| b)
                    .ok_or_else(|| format!("`'{name}'` was never declared"))?;
                if !bound.mutable {
                    return Err(format!(
                        "`'{name}'` does not change. A chain says `mut` where a name is allowed to."
                    ));
                }
                bound.value = value;
            }
            Stmt::Print(items) => {
                for item in items {
                    match item {
                        Item::Text(text) => out.push_str(text),
                        Item::Newline => out.push('\n'),
                        Item::Value(expr) => out.push_str(&eval(expr, names)?.to_string()),
                    }
                }
            }
            Stmt::Loop { name, from, to, body } => {
                let from = eval(from, names)?;
                let to = eval(to, names)?;
                let depth = names.len();
                let mut i = from;
                while i <= to {
                    names.truncate(depth);
                    names.push((name.clone(), Bound { value: i, mutable: false }));
                    perform(body, names, out, steps)?;
                    *steps += 1;
                    if *steps > MOST_STEPS {
                        return Err("the playground stopped: that is more work than it will do in a browser".into());
                    }
                    i += 1;
                }
                names.truncate(depth);
            }
        }
    }
    Ok(())
}

fn eval(expr: &Expr, names: &[(String, Bound)]) -> Result<i64, String> {
    Ok(match expr {
        Expr::Num(n) => *n,
        Expr::Name(name) => {
            names
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, b)| b.value)
                .ok_or_else(|| format!("`'{name}'` was never declared"))?
        }
        Expr::Bin(left, op, right) => {
            let (a, b) = (eval(left, names)?, eval(right, names)?);
            let outcome = match op {
                Op::Add => a.checked_add(b),
                Op::Sub => a.checked_sub(b),
                Op::Mul => a.checked_mul(b),
                Op::Div => {
                    if b == 0 {
                        return Err("dividing by nothing".into());
                    }
                    a.checked_div(b)
                }
                Op::Mod => {
                    if b == 0 {
                        return Err("the remainder of dividing by nothing".into());
                    }
                    a.checked_rem(b)
                }
            };
            outcome.ok_or("that goes outside what an `int64` holds")?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ran(src: &str) -> String {
        run(src).unwrap_or_else(|e| panic!("{src}\n  -> {e}"))
    }

    #[test]
    fn it_prints() {
        assert_eq!(ran("START { print.stdout[str:*hi* \\n]; }"), "hi\n");
    }

    #[test]
    fn arithmetic_is_the_arithmetic_the_compiler_does() {
        let out = ran(
            "START {\n\
               var.int64 'a' = [*2* + *3* x *4*];\n\
               var.int64 'b' = [*7* / *2*];\n\
               var.int64 'c' = [(*0* - *7*) / *2*];\n\
               var.int64 'd' = [(*0* - *7*) mod *3*];\n\
               print.stdout['a' str:*,* 'b' str:*,* 'c' str:*,* 'd' \\n];\n\
             }",
        );
        assert_eq!(out, "14,3,-3,-1\n");
    }

    /// The one rule the language is loudest about: an order nobody agreed on
    /// is refused rather than picked.
    #[test]
    fn mod_beside_plus_is_refused() {
        let err = run("START { var.int64 'a' = [*9* mod *5* + *1*]; }").unwrap_err();
        assert!(err.contains("no agreed order"), "{err}");
        assert!(run("START { var.int64 'a' = [(*9* mod *5*) + *1*]; print.stdout['a' \\n]; }").is_ok());
    }

    #[test]
    fn a_loop_runs_over_both_ends() {
        let out = ran(
            "START {\n\
               var.mut.int64 'n' = [*0*];\n\
               loop.range.int64 'i' = [*1*, *3*] { set 'n' = ['n' + 'i']; }\n\
               print.stdout['n' \\n];\n\
             }",
        );
        assert_eq!(out, "6\n");
    }

    /// `const` is declared above `START`, not in it — which the playground had
    /// wrong until the compiler was asked.
    #[test]
    fn const_lives_above_the_start_block() {
        let out = ran(
            "const.int64 'LIMIT' = [*4*];\n\
             START { print.stdout['LIMIT' \\n]; }",
        );
        assert_eq!(out, "4\n");

        let err = run("START { const.int64 'a' = [*1*]; }").unwrap_err();
        assert!(err.contains("outside `START`"), "{err}");
    }

    #[test]
    fn a_name_without_mut_does_not_change() {
        let err = run("START { var.int64 'a' = [*1*]; set 'a' = [*2*]; }").unwrap_err();
        assert!(err.contains("does not change"), "{err}");
    }

    #[test]
    fn a_written_value_in_a_print_list_says_its_type() {
        let err = run("START { print.stdout[*5* \\n]; }").unwrap_err();
        assert!(err.contains("has no type here"), "{err}");
        assert_eq!(ran("START { print.stdout[int64:*5* \\n]; }"), "5\n");
    }

    #[test]
    fn what_it_does_not_know_it_says_so_about() {
        for (src, expected) in [
            ("START { var.str 'a' = [*hi*]; }", "only knows `int64`"),
            ("START { give [*1*]; }", "does not know `give`"),
            ("START { var.int64 'a' = [*1* / *0*]; }", "dividing by nothing"),
            ("START { print.stdout['nope' \\n]; }", "never declared"),
        ] {
            let err = run(src).unwrap_err();
            assert!(err.contains(expected), "{src}\n  got: {err}");
        }
    }

    /// A browser must not be handed a program that never stops.
    #[test]
    fn a_loop_that_will_not_end_is_stopped() {
        let err = run(
            "START { loop.range.int64 'i' = [*1*, *999999999*] { var.int64 'a' = [*1*]; } }",
        )
        .unwrap_err();
        assert!(err.contains("more work than it will do"), "{err}");
    }

    #[test]
    fn the_lists_of_what_it_is_and_is_not_are_both_real() {
        assert!(!SUPPORTED.is_empty() && !UNSUPPORTED.is_empty());
        assert!(UNSUPPORTED.len() >= SUPPORTED.len(), "the honest list must not be the shorter one");
    }
}
