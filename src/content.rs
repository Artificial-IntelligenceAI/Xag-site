//! Everything the site says, in one place.
//!
//! This is the site's prose as data rather than as markup, because it has to be
//! rendered twice: once into HTML at build time, for anything that will not run
//! WebAssembly, and once by the app. Two copies of a sentence is two sentences
//! that will disagree eventually, so there is one, here.
//!
//! Prose is written in the small markup in `markup.rs`. Programs and compiler
//! output are `include_str!`, so they are the files that were run rather than
//! transcriptions of them.

pub const REPO: &str = "https://github.com/Artificial-IntelligenceAI/Xag-lang";
pub const SITE_REPO: &str = "https://github.com/Artificial-IntelligenceAI/Xag-site";

/// One thing on a page.
pub enum Block {
    /// A paragraph.
    Para(&'static str),
    /// A line worth stopping on.
    Claim(&'static str),
    /// Set apart, and about the state of things rather than the language.
    Warning(&'static str),
    /// Xag source, coloured by the tokeniser.
    Code(&'static str),
    /// Commands to run in a shell.
    Shell(&'static str),
    /// What the compiler actually said, verbatim.
    Diagnostic(&'static str),
    /// A program that was run, and what it printed.
    Sample {
        file: &'static str,
        src: &'static str,
        output: &'static str,
    },
    /// The three things a piece of Xag can be.
    MarksTable,
    /// The panel that colours what you type. Interactive, so the built HTML
    /// leaves a note where it would be rather than pretending to have one.
    LivePanel,
    /// A list where each item leads with a name.
    Bullets(&'static [(&'static str, &'static str)]),
    /// Somebody else's words.
    Quote {
        paragraphs: &'static [&'static str],
        attribution: &'static str,
    },
}

pub struct Section {
    pub id: &'static str,
    pub heading: &'static str,
    pub blocks: &'static [Block],
}

pub struct Page {
    /// The last part of the address. Empty is the front page.
    pub slug: &'static str,
    /// What the browser tab and a search result call it.
    pub title: &'static str,
    /// One sentence, for a description and a link preview.
    pub summary: &'static str,
    /// The opening paragraph, above the first heading.
    pub lede: &'static str,
    pub sections: &'static [Section],
}

pub const PAGES: [Page; 3] = [HOME, PHILOSOPHY, CREDITS];

pub fn page(slug: &str) -> &'static Page {
    match slug {
        "philosophy" => &PHILOSOPHY,
        "credits" => &CREDITS,
        _ => &HOME,
    }
}

// ─────────────────────────────── Home ───────────────────────────────

pub const HOME: Page = Page {
    slug: "",
    title: "Xag",
    summary: "A compiled programming language with Rust-style ownership checked \
              at compile time. A size is always written: there is no int on its \
              own, because there is no size to assume.",
    lede: "A programming language built around high performance, helpful error \
           messages, and Rust-style memory management. There is no `int` on its own, \
           because there is no size to assume. Programs are compiled ahead of time, \
           either to native code or to a form run by an AOT interpreter.",
    sections: &[
        Section {
            id: "reading",
            heading: "Reading a program",
            blocks: &[
                Block::Para(
                    "Items sit next to each other and are used in order. Nothing is \
                     concatenated into a third thing, so there is no `+` for text.",
                ),
                Block::Sample {
                    file: "counting.xag",
                    src: include_str!("../samples/counting.xag"),
                    output: "sum to 10 = 55",
                },
            ],
        },
        Section {
            id: "ownership",
            heading: "What a name owns, and what it lends",
            blocks: &[
                Block::Para(
                    "Ownership is Rust's, checked when the program is compiled, with \
                     no garbage collector under it. A transfer is spelled at the call \
                     site, so a reader never has to work out from a function's \
                     signature that a value has left.",
                ),
                Block::Sample {
                    file: "borrowing.xag",
                    src: include_str!("../samples/borrowing.xag"),
                    output: "size: 5\nafter: hello!\nlonger: hello!\nkept: spare",
                },
            ],
        },
        Section {
            id: "errors",
            heading: "When it has something to say",
            blocks: &[
                Block::Para(
                    "Take the program above, hand a value over for good, and then ask \
                     for it again. This is what comes back — not a paraphrase of it:",
                ),
                Block::Diagnostic(include_str!("../samples/moved.output.txt")),
                Block::Para(
                    "A diagnostic that points at the wrong thing, or names a rule the \
                     program did not break, is a bug of the same kind as miscompiling \
                     — the compiler is telling the reader something untrue either way.",
                ),
            ],
        },
        Section {
            id: "building",
            heading: "Building it",
            blocks: &[
                Block::Para(
                    "The compiler is written in C++20 against LLVM's native C++ API. \
                     It needs LLVM 23 or newer, CMake and Ninja.",
                ),
                Block::Shell(
                    "git clone https://github.com/Artificial-IntelligenceAI/Xag-lang.git\n\
                     cd Xag-lang\n\
                     cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release\n\
                     ninja -C build\n\
                     ./build/xagc run examples/counting.xag",
                ),
                Block::Para(
                    "`xagc run` runs a program on the test interpreter, `xagc fast` on \
                     the fast one, and `xagc build` compiles it ahead of time and \
                     writes an executable beside it. `xagc check` checks a program \
                     without running it.",
                ),
                Block::Warning(
                    "There is no install yet. Building from source is the way to run it.",
                ),
            ],
        },
    ],
};

// ──────────────────────────── Philosophy ────────────────────────────

pub const PHILOSOPHY: Page = Page {
    slug: "philosophy",
    title: "Design Philosophy — Xag",
    summary: "Why Xag is written the way it is: two marks and only two, \
              declarations as chains, and three engines kept apart on purpose.",
    lede: "What is unusual about Xag is not what it can do. It is what it refuses \
           to decide on your behalf.",
    sections: &[
        Section {
            id: "marks",
            heading: "Two marks, and only two",
            blocks: &[
                Block::MarksTable,
                Block::Para(
                    "A quoted thing is a name wherever you meet it. It never has to be \
                     re-read as a value because of where it happens to sit — position \
                     is never consulted.",
                ),
                Block::Para(
                    "Every declaration marks the name it gives, including a function's \
                     and a struct's. What is being named is marked; what is being used \
                     is whatever it was declared as.",
                ),
                Block::Code("fn.int64 'sum-to' [int64 'n'] { give ['n']; }"),
                Block::Para(
                    "There is no third mark for text versus number, because the type \
                     already answers that: `*1000*`{value} is a number under `int64` \
                     and four characters under `str`.",
                ),
                Block::Claim("There are only two marks and there will only ever be two."),
                Block::LivePanel,
            ],
        },
        Section {
            id: "chains",
            heading: "Declarations are chains",
            blocks: &[
                Block::Para(
                    "What is unusual about a name lives in the chain that declares it. \
                     Every segment but the type has a default, and the default is \
                     always the least powerful thing — so a word appears only where \
                     there was a choice.",
                ),
                Block::Code("var.mut.many.int64 'xs'"),
                Block::Para(
                    "That one changes, holds several, and holds 64-bit whole numbers. \
                     Drop `mut` and it does not change. Drop `many` and it holds one.",
                ),
            ],
        },
        Section {
            id: "engines",
            heading: "Three engines and an oracle",
            blocks: &[
                Block::Para(
                    "There are three ways to run an Xag program, and they are kept \
                     apart on purpose.",
                ),
                Block::Bullets(&[
                    (
                        "A test interpreter",
                        "built to be obviously correct rather than fast. It walks the \
                         IR as written and does nothing clever anywhere. It is the one \
                         to believe when the engines disagree.",
                    ),
                    (
                        "A fast interpreter",
                        "it turns the graph into flat code once and then runs it \
                         without looking anything up again. It shares nothing with the \
                         test interpreter but the runtime, on purpose: two engines that \
                         borrow from each other agree about what they borrowed, and a \
                         vote between them proves nothing.",
                    ),
                    ("A native backend", "compiled ahead of time through LLVM."),
                ]),
                Block::Claim("Two engines can say that something is wrong; three can say which."),
                Block::Para(
                    "A program generator writes random Xag programs, asks every engine \
                     what they say, and when they differ reports which one is out of \
                     step with the other two.",
                ),
                Block::Para(
                    "Decimal gets a check the oracle cannot give it. Three engines \
                     calling one runtime agree about everything inside that runtime, so \
                     a mistake in the arithmetic itself is one all three would make \
                     together. So the decimal results are checked against Python's \
                     `decimal` — libmpdec, written by someone else from the same IBM \
                     specification, and derived from nothing here. Four hundred \
                     thousand cases agree exactly, apart from raising to a power, which \
                     the specification itself allows to be out by one in the last place.",
                ),
                Block::Para(
                    "They are also asked of a real decimal floating-point unit. No \
                     machine here has one, so the test cross-compiles a program with no \
                     operating system under it, hands it to QEMU where a kernel would \
                     go, and reads the answers back over the console. Twenty thousand \
                     sums, differences, products and quotients agree exactly, cohorts \
                     included.",
                ),
            ],
        },
    ],
};

// ────────────────────────────── Credits ──────────────────────────────

pub const CREDITS: Page = Page {
    slug: "credits",
    title: "Credits, License, Source — Xag",
    summary: "Where Xag's source is, what it is licensed under, and who made it.",
    lede: "",
    sections: &[
        Section {
            id: "honest",
            heading: "Why would anyone ever use Xag?",
            blocks: &[Block::Quote {
                paragraphs: &[
                    "Honestly, I don't know. 😂",
                    "The code written is mostly or 100% AI-made. Why? Because, I'm \
                     more of a designer, not a C++ stroke-inducing syntax reader 🤣. \
                     No offense.",
                ],
                attribution: "— the README",
            }],
        },
        Section {
            id: "source",
            heading: "Source",
            blocks: &[
                Block::Para(
                    "The compiler, the runtime, the three engines and the oracle: \
                     [github.com/Artificial-IntelligenceAI/Xag-lang]\
                     (https://github.com/Artificial-IntelligenceAI/Xag-lang)",
                ),
                Block::Para(
                    "A diagnostic that is wrong, a diagnostic that is confusing, and a \
                     program Xag accepts that it should not are all the same kind of \
                     bug: the compiler saying something untrue. They belong in [the \
                     issue tracker]\
                     (https://github.com/Artificial-IntelligenceAI/Xag-lang/issues).",
                ),
                Block::Para(
                    "This site is a Rust program compiled to WebAssembly. Its source is \
                     [github.com/Artificial-IntelligenceAI/Xag-site]\
                     (https://github.com/Artificial-IntelligenceAI/Xag-site)",
                ),
            ],
        },
        Section {
            id: "license",
            heading: "License",
            blocks: &[
                Block::Para("Copyright 2026 Tankun Sriket"),
                Block::Para("Licensed under either of"),
                Block::Bullets(&[
                    (
                        "Apache License, Version 2.0",
                        "[apache.org/licenses/LICENSE-2.0]\
                         (https://www.apache.org/licenses/LICENSE-2.0)",
                    ),
                    (
                        "MIT license",
                        "[opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)",
                    ),
                ]),
                Block::Para("at your option."),
                Block::Para(
                    "Unless you explicitly state otherwise, any contribution \
                     intentionally submitted for inclusion in this project by you, as \
                     defined in the Apache-2.0 license, shall be dual licensed as \
                     above, without any additional terms or conditions.",
                ),
            ],
        },
        Section {
            id: "credits",
            heading: "Credits",
            blocks: &[
                Block::Para(
                    "Xag is designed by human Tankun Sriket, and implemented by AI \
                     (Anthropic's Claude). The compiler is written against LLVM, and \
                     its grapheme handling is built from the Unicode Character \
                     Database, version 17.0.0.",
                ),
                Block::Para(
                    "The decimal arithmetic is checked against Python's `decimal` — \
                     libmpdec, by Stefan Krah — and against IBM's decimal \
                     floating-point unit under QEMU. Neither is derived from anything \
                     here, which is what makes them worth asking.",
                ),
            ],
        },
    ],
};

/// The three things a piece of Xag can be, and what each is.
pub const MARKS: [(&str, &str, &str); 3] = [
    ("tk-name", "'…'", "a **name**"),
    ("tk-value", "*…*", "a **written value**"),
    ("tk-word", "word", "a type, a segment of a chain, or a function being called"),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every page has to be reachable and describable, or the built HTML has
    /// nothing to put in a title or a description.
    #[test]
    fn every_page_says_what_it_is() {
        for page in PAGES.iter() {
            assert!(!page.title.is_empty(), "a page has no title");
            assert!(!page.summary.is_empty(), "{} has no summary", page.title);
            assert!(!page.sections.is_empty(), "{} has no sections", page.title);
            for section in page.sections {
                assert!(!section.id.is_empty());
                assert!(!section.heading.is_empty());
                assert!(!section.blocks.is_empty(), "{} is empty", section.heading);
            }
        }
    }

    /// A card or a link pointing at a section that is not there would scroll
    /// nowhere, and nothing else would notice.
    #[test]
    fn the_sections_the_cards_point_at_exist() {
        let ids: Vec<&str> = PAGES
            .iter()
            .flat_map(|p| p.sections.iter().map(|s| s.id))
            .collect();
        for wanted in ["marks", "ownership", "errors", "engines", "building"] {
            assert!(ids.contains(&wanted), "no section with id {wanted}");
        }
    }

    #[test]
    fn a_slug_finds_its_page() {
        assert_eq!(page("philosophy").slug, "philosophy");
        assert_eq!(page("credits").slug, "credits");
        assert_eq!(page("").slug, "");
        assert_eq!(page("nonsense").slug, "", "unknown slugs land on the front page");
    }
}
