//! Writes the site as HTML, for whatever will not run WebAssembly.
//!
//! What comes out is the same words the app renders, from the same data, laid
//! out with the same class names so the same stylesheet dresses it. It is not
//! the app: there is no sky, no theme picker and no panel that colours what you
//! type. It is the reading half, which is the half a crawler, a reader with
//! scripting off, and anything fetching a URL for a language model wants.
//!
//! The app mounts over it and replaces it, so a browser gets the words first
//! and the rest a moment later.

use crate::content::{Block, Page, Section, MARKS};
use crate::markup::{escape, to_html, to_text};
use crate::syntax::tokenize;

/// Xag source, coloured the way the app colours it — the same tokeniser, so a
/// name is the same green before the WebAssembly arrives as after.
pub fn highlight(src: &str) -> String {
    tokenize(src)
        .into_iter()
        .map(|t| format!("<span class=\"{}\">{}</span>", t.kind.class(), escape(&t.text)))
        .collect()
}

fn block(b: &Block, out: &mut String) {
    match b {
        Block::Para(text) => out.push_str(&format!("<p>{}</p>", to_html(text))),
        Block::Claim(text) => out.push_str(&format!("<p class=\"claim\">{}</p>", to_html(text))),
        Block::Warning(text) => {
            out.push_str(&format!("<p class=\"warning\">{}</p>", to_html(text)))
        }
        Block::Code(src) => out.push_str(&format!(
            "<pre class=\"code\"><code>{}</code></pre>",
            highlight(src)
        )),
        Block::Shell(src) => out.push_str(&format!(
            "<pre class=\"shell\"><code>{}</code></pre>",
            escape(src)
        )),
        Block::Diagnostic(text) => out.push_str(&format!(
            "<pre class=\"diagnostic\"><code>{}</code></pre>",
            escape(text)
        )),
        Block::Sample { file, src, output } => {
            out.push_str(&format!(
                "<figure class=\"sample\">\
                   <pre class=\"code\"><code>{}</code></pre>\
                   <figcaption><span class=\"filename\">{file}</span>\
                     <span class=\"sep\"> — </span>\
                     <code class=\"cmd\">xagc run {file}</code></figcaption>\
                   <pre class=\"output\"><code>{}</code></pre>\
                 </figure>",
                highlight(src),
                escape(output)
            ));
        }
        Block::Questions => {
            for q in crate::content::QUESTIONS.iter() {
                out.push_str(&format!(
                    "<h3 id=\"{}\">{}</h3><p class=\"claim\">{}</p>",
                    q.id,
                    escape(q.asked),
                    to_html(q.short)
                ));
                for para in q.answer.iter() {
                    out.push_str(&format!("<p>{}</p>", to_html(para)));
                }
            }
        }
        Block::MarksTable => {
            out.push_str("<table class=\"marks-table\"><tbody>");
            for (class, mark, meaning) in MARKS.iter() {
                out.push_str(&format!(
                    "<tr><td><code class=\"{class}\">{}</code></td><td>{}</td></tr>",
                    escape(mark),
                    to_html(meaning)
                ));
            }
            out.push_str("</tbody></table>");
        }
        // The panel is the one thing here that cannot be written down: it
        // answers as you type. Saying so is better than leaving a hole.
        Block::LivePanel => out.push_str(
            "<p class=\"marks-note\">There is a panel here that colours Xag as you \
             type it, once the page has loaded.</p>",
        ),
        Block::Bullets(items) => {
            out.push_str("<ul class=\"engines\">");
            for (lead, rest) in items.iter() {
                out.push_str(&format!(
                    "<li><strong>{}</strong> — {}</li>",
                    to_html(lead),
                    to_html(rest)
                ));
            }
            out.push_str("</ul>");
        }
        Block::Quote {
            paragraphs,
            attribution,
        } => {
            out.push_str("<blockquote>");
            for p in paragraphs.iter() {
                out.push_str(&format!("<p>{}</p>", to_html(p)));
            }
            out.push_str(&format!("<footer>{}</footer></blockquote>", to_html(attribution)));
        }
    }
}

/// The body of one page: what goes inside `main`.
pub fn body(page: &Page) -> String {
    let mut out = String::with_capacity(4096);
    if !page.lede.is_empty() {
        out.push_str(&format!("<p class=\"lede\">{}</p>", to_html(page.lede)));
    }
    for Section { id, heading, blocks } in page.sections.iter() {
        out.push_str(&format!("<section id=\"{id}\"><h2>{}</h2>", escape(heading)));
        for b in blocks.iter() {
            block(b, &mut out);
        }
        out.push_str("</section>");
    }
    out
}

/// A whole document, ready to be served.
///
/// `theme` is the attribute the address asked for, so a reader without
/// WebAssembly still gets the palette they typed rather than the default one.
pub fn document(
    page: &Page,
    theme: Option<&str>,
    canonical: &str,
    css: &str,
    js: &str,
    wasm: &str,
) -> String {
    let theme_attr = theme.map(|t| format!(" data-theme=\"{t}\"")).unwrap_or_default();
    format!(
        r#"<!DOCTYPE html>
<html lang="en"{theme_attr}>
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
<meta name="description" content="{summary}" />
<link rel="canonical" href="{canonical}" />
<meta name="color-scheme" content="dark light" />
<meta property="og:title" content="{title}" />
<meta property="og:description" content="{summary}" />
<meta property="og:type" content="website" />
<meta property="og:url" content="{canonical}" />
<link rel="stylesheet" href="/{css}" />
</head>
<body>
<div id="fallback">
<header class="hero{compact}"><div class="hero-inner">
<div class="hero-mark"><h1><span class="wordmark">Xag</span></h1></div>
<div class="hero-words">
{tagline}
<p class="warning">Under development, don&#39;t treat this website&#39;s info as <a href="{repo}">truth</a>.</p>
</div></div></header>
<main>{body}</main>
<nav class="dock" aria-label="Sections">{nav}</nav>
</div>
<script type="module">
// The module path is given rather than inferred: with nothing passed, the
// loader derives an unhashed name and asks for a file that is not there.
import init from '/{js}';
init({{ module_or_path: '/{wasm}' }});
</script>
</body>
</html>
"#,
        title = escape(page.title),
        summary = escape(&to_text(page.summary)),
        canonical = escape(canonical),
        repo = crate::content::REPO,
        body = body(page),
        // The front page stands full height and says what the language is like;
        // the others have already been arrived at.
        compact = if page.slug.is_empty() { "" } else { " compact" },
        tagline = if page.slug.is_empty() { tagline() } else { String::new() },
        nav = nav(page.slug, theme),
        css = css,
        js = js,
        wasm = wasm,
    )
}

/// What the language is like, on the front page only — the others are already
/// somewhere, and do not need telling what they arrived at.
fn tagline() -> String {
    let mut out = String::from("<p class=\"tagline\">");
    for (text, good) in crate::content::TAGLINE.iter() {
        let class = if *good { "good" } else { "cost" };
        out.push_str(&format!("<span class=\"{class}\">{}</span>", escape(text)));
    }
    out.push_str("</p>");
    out
}

/// Links to the other pages, so a crawler can find them and a reader without
/// WebAssembly can still move around.
fn nav(here: &str, theme: Option<&str>) -> String {
    let prefix = match theme {
        Some("solarized") => "/lite",
        Some("solarized-light") => "/lite2",
        Some("silver") => "/silver",
        Some(_) | None => "/alien",
    };
    let mut out = String::new();
    for page in crate::content::PAGES.iter() {
        let href = if page.slug.is_empty() {
            prefix.to_string()
        } else {
            format!("{prefix}/{}", page.slug)
        };
        let label = match page.slug {
            "" => "Home",
            "questions" => "Questions",
            "philosophy" => "Design Philosophy",
            "credits" => "Credits, License, Source",
            other => other,
        };
        let here = if page.slug == here { " here" } else { "" };
        out.push_str(&format!(
            "<a class=\"dock-item{here}\" href=\"{}\">{label}</a>",
            escape(&href)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{CREDITS, HOME, PAGES, PHILOSOPHY};

    /// The whole point: the words are in the HTML rather than only in the
    /// WebAssembly.
    #[test]
    fn the_prose_is_in_the_document() {
        let doc = document(&HOME, None, "https://xag-lang.com/", "s.css", "a.js", "a_bg.wasm");
        assert!(doc.contains("Items sit next to each other"));
        assert!(doc.contains("There is no install yet"));
        // And the programs, which were run rather than transcribed.
        assert!(doc.contains("sum to 10 = 55"));
        assert!(doc.contains("was moved, and holds nothing now"));
    }

    #[test]
    fn each_page_says_what_it_is_rather_than_what_the_last_one_was() {
        let a = document(&HOME, None, "https://xag-lang.com/alien", "s.css", "a.js", "a_bg.wasm");
        let b = document(&PHILOSOPHY, None, "https://xag-lang.com/alien/philosophy", "s.css", "a.js", "a_bg.wasm");
        assert!(a.contains("<title>Xag</title>"));
        assert!(b.contains("<title>Design Philosophy — Xag</title>"));
        assert_ne!(a, b, "two pages served the same document");
    }

    /// Every page has to link to every other, or a crawler arriving at one has
    /// no way to the rest.
    #[test]
    fn every_page_links_to_every_other() {
        for page in PAGES.iter() {
            let doc = document(page, None, "https://xag-lang.com/", "s.css", "a.js", "a_bg.wasm");
            for other in PAGES.iter() {
                let href = if other.slug.is_empty() {
                    "/alien".to_string()
                } else {
                    format!("/alien/{}", other.slug)
                };
                assert!(doc.contains(&format!("href=\"{href}\"")), "{} does not link to {href}", page.title);
            }
        }
    }

    #[test]
    fn the_theme_in_the_address_is_in_the_document() {
        let doc = document(&CREDITS, Some("solarized-light"), "https://xag-lang.com/lite2/credits", "s.css", "a.js", "a_bg.wasm");
        assert!(doc.contains("data-theme=\"solarized-light\""));
        assert!(doc.contains("href=\"/lite2/philosophy\""), "the nav keeps the theme");
    }

    /// Xag in the served HTML is coloured by the same tokeniser the app uses.
    #[test]
    fn the_code_is_coloured_before_the_app_arrives() {
        let doc = document(&PHILOSOPHY, None, "https://xag-lang.com/", "s.css", "a.js", "a_bg.wasm");
        assert!(doc.contains("<span class=\"tk-name\">&#39;sum-to&#39;</span>"));
    }

    #[test]
    fn nothing_unescaped_gets_into_the_markup() {
        let doc = document(&HOME, None, "https://xag-lang.com/", "s.css", "a.js", "a_bg.wasm");
        // The diagnostic contains backticks and carets; none of it may become a
        // tag. Only what is between the main tags — the document's own script
        // element sits after them and is meant to be there.
        let main = doc.split("<main>").nth(1).unwrap().split("</main>").next().unwrap();
        assert!(!main.contains("<script"), "something got through");

        // The compiler's own output arrives as text, carets and all, rather
        // than as anything a browser would try to interpret.
        assert!(main.contains("^^^^^^^^^^"), "the diagnostic lost its carets");

        // And an angle bracket in prose stays an angle bracket.
        assert_eq!(
            crate::markup::to_html("a <b> in a sentence"),
            "a &lt;b&gt; in a sentence"
        );
    }
}
