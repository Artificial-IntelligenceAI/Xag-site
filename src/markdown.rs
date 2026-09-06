//! The whole site as one plain-text document.
//!
//! `llms.txt` is an index: it says what the pages are and where. This is the
//! other half of that convention — everything the site says, in one fetch, for
//! anything reading rather than looking.
//!
//! It is generated from the same `content.rs` as the HTML and the app, so there
//! is still one copy of every sentence. A hand-written summary of a site is a
//! summary of the site as it was the day somebody wrote it.

use crate::content::{Block, Page, Section, MARKS, PAGES, REPO, SITE_REPO};
use crate::markup::{to_markdown, to_text};

fn block(b: &Block, out: &mut String) {
    match b {
        Block::Para(text) => out.push_str(&format!("{}\n\n", to_markdown(text))),
        Block::Claim(text) => out.push_str(&format!("> {}\n\n", to_markdown(text))),
        Block::Warning(text) => out.push_str(&format!("**{}**\n\n", to_markdown(text))),
        Block::Code(src) => out.push_str(&format!("```xag\n{src}\n```\n\n")),
        Block::Shell(src) => out.push_str(&format!("```sh\n{src}\n```\n\n")),
        Block::Diagnostic(text) => out.push_str(&format!(
            "What the compiler says, verbatim:\n\n```text\n{}\n```\n\n",
            text.trim_end()
        )),
        Block::Sample { file, src, output } => out.push_str(&format!(
            "`{file}`, run with `xagc run {file}`:\n\n```xag\n{}\n```\n\nIt prints:\n\n```text\n{output}\n```\n\n",
            src.trim_end()
        )),
        Block::Questions => {
            for q in crate::content::QUESTIONS.iter() {
                out.push_str(&format!("**{}**\n\n{}\n\n", q.asked, to_markdown(q.short)));
                for para in q.answer.iter() {
                    out.push_str(&format!("{}\n\n", to_markdown(para)));
                }
            }
        }
        Block::PlainList(items) => {
            for item in items.iter() {
                out.push_str(&format!("- {}\n", to_markdown(item)));
            }
            out.push('\n');
        }
        Block::Playground => out.push_str(
            "*(There is an editor here on the page, running a small interpreter in \
             the browser. It is not the compiler.)*\n\n",
        ),
        Block::MarksTable => {
            out.push_str("| written | is |\n| --- | --- |\n");
            for (_, mark, meaning) in MARKS.iter() {
                out.push_str(&format!("| `{mark}` | {} |\n", to_markdown(meaning)));
            }
            out.push('\n');
        }
        Block::LivePanel => out.push_str(
            "*(On the page here there is a panel that colours Xag as you type it. \
             It colours; it does not run.)*\n\n",
        ),
        Block::Bullets(items) => {
            for (lead, rest) in items.iter() {
                out.push_str(&format!("- **{lead}** — {}\n", to_markdown(rest)));
            }
            out.push('\n');
        }
        Block::Quote { paragraphs, attribution } => {
            for p in paragraphs.iter() {
                out.push_str(&format!("> {}\n>\n", to_markdown(p)));
            }
            out.push_str(&format!("> {attribution}\n\n"));
        }
    }
}

/// One page, as Markdown, under a heading of its own.
pub fn page(page: &Page, site: &str) -> String {
    let mut out = String::with_capacity(4096);
    let address = if page.slug.is_empty() {
        format!("{site}/alien")
    } else {
        format!("{site}/alien/{}", page.slug)
    };
    out.push_str(&format!("## {}\n\n", to_text(page.title)));
    out.push_str(&format!("Source: {address}\n\n"));
    if !page.lede.is_empty() {
        out.push_str(&format!("{}\n\n", to_markdown(page.lede)));
    }
    for Section { heading, blocks, .. } in page.sections.iter() {
        out.push_str(&format!("### {heading}\n\n"));
        for b in blocks.iter() {
            block(b, &mut out);
        }
    }
    out
}

/// The whole site.
pub fn everything(site: &str) -> String {
    let mut out = String::with_capacity(16_384);
    out.push_str("# Xag\n\n");
    out.push_str(&format!("> {}\n\n", to_text(PAGES[0].summary)));
    out.push_str(
        "Xag is early and unstable. **The compiler is the authority on what the \
         language does; this website is not.** Anything here may be out of date. \
         Every program below was run against the compiler before it was written \
         down, and the compiler output is what it printed rather than a \
         transcription — but that was true on the day it was built, and the \
         language moves.\n\n",
    );
    out.push_str(&format!(
        "- Compiler, runtime, three engines and oracle: {REPO}\n\
         - This website: {SITE_REPO}\n\n---\n\n"
    ));
    for p in PAGES.iter() {
        out.push_str(&page(p, site));
        out.push_str("---\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_means_everything() {
        let md = everything("https://xag-lang.com");
        // A sentence from each of the three pages.
        assert!(md.contains("Items sit next to each other"));
        assert!(md.contains("position is never consulted"));
        assert!(md.contains("Copyright 2026 Tankun Sriket"));
        // And the programs, which were run rather than described.
        assert!(md.contains("sum to 10 = 55"));
        assert!(md.contains("was moved, and holds nothing now"));
    }

    /// It has to say what it is not, or it is a confident-sounding document
    /// about a language that may have moved.
    #[test]
    fn it_says_the_compiler_is_the_authority() {
        let md = everything("https://xag-lang.com");
        assert!(md.contains("The compiler is the authority"));
        assert!(md.contains("github.com/Artificial-IntelligenceAI/Xag-lang"));
    }

    #[test]
    fn every_page_is_in_it_and_addressed() {
        let md = everything("https://xag-lang.com");
        for p in PAGES.iter() {
            assert!(md.contains(&to_text(p.title)), "{} is missing", p.title);
        }
        assert!(md.contains("https://xag-lang.com/alien/philosophy"));
        assert!(md.contains("https://xag-lang.com/alien/credits"));
    }

    /// Markdown, not HTML: the colouring is a fact about the page and means
    /// nothing in a text file.
    #[test]
    fn no_html_and_no_colours_leak_in() {
        let md = everything("https://xag-lang.com");
        assert!(!md.contains("<code"), "HTML got in");
        assert!(!md.contains("{value}"), "a colour got in");
        assert!(!md.contains("tk-name"), "a class name got in");
    }
}
