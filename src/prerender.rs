//! Writes the HTML that is served before the WebAssembly has loaded.
//!
//! Trunk builds `dist/` with one `index.html` for every address, which is
//! enough for the app — it reads the path and decides — but means every address
//! serves the same hundred words to anything that will not run WebAssembly.
//! This writes a real document for each one instead, with that page's prose, its
//! own title, and the palette the address asked for.
//!
//! It runs after Trunk, over what Trunk produced, and takes the asset names out
//! of the `index.html` Trunk wrote so the hashes always match what was built.
//!
//!     cargo run --features prerender --bin prerender -- dist https://xag-lang.com

use std::fs;
use std::path::{Path, PathBuf};

use xag_site::content::PAGES;
use xag_site::html;
use xag_site::route::THEMES;

fn main() {
    let mut args = std::env::args().skip(1);
    let dist = PathBuf::from(args.next().unwrap_or_else(|| "dist".into()));
    let site = args
        .next()
        .unwrap_or_else(|| "https://xag-lang.com".into())
        .trim_end_matches('/')
        .to_string();

    let index = dist.join("index.html");
    let built = fs::read_to_string(&index)
        .unwrap_or_else(|e| fail(&format!("cannot read {}: {e}", index.display())));

    let css = asset(&built, ".css").unwrap_or_else(|| fail("no stylesheet in dist/index.html"));
    let js = asset(&built, ".js").unwrap_or_else(|| fail("no script in dist/index.html"));
    let wasm = asset(&built, ".wasm").unwrap_or_else(|| fail("no wasm in dist/index.html"));

    let mut written = 0usize;

    // Every address the site answers to, and what it should say.
    for theme in THEMES {
        for page in PAGES.iter() {
            let path = if page.slug.is_empty() {
                format!("{}", theme.slug())
            } else {
                format!("{}/{}", theme.slug(), page.slug)
            };
            write(&dist, &path, &html::document(
                page,
                theme.attribute(),
                &format!("{site}/{}", canonical_of(page.slug)),
                &css,
                &js,
                &wasm,
            ));
            written += 1;
        }
    }

    // The same pages without a theme in front, which the app rewrites but which
    // somebody may still link to.
    for page in PAGES.iter().filter(|p| !p.slug.is_empty()) {
        write(&dist, page.slug, &html::document(
            page,
            None,
            &format!("{site}/{}", canonical_of(page.slug)),
            &css,
            &js,
            &wasm,
        ));
        written += 1;
    }

    // And the root, which decides its theme from the domain it was asked on.
    fs::write(&index, html::document(
        &PAGES[0],
        None,
        &format!("{site}/"),
        &css,
        &js,
        &wasm,
    ))
    .unwrap_or_else(|e| fail(&format!("cannot write {}: {e}", index.display())));
    written += 1;

    crawler_files(&dist, &site);

    println!("prerendered {written} documents into {}", dist.display());
}

/// The address a page should be indexed under, whichever one was asked for.
fn canonical_of(slug: &str) -> String {
    if slug.is_empty() {
        "alien".into()
    } else {
        format!("alien/{slug}")
    }
}

fn write(dist: &Path, path: &str, body: &str) {
    let dir = dist.join(path);
    fs::create_dir_all(&dir).unwrap_or_else(|e| fail(&format!("cannot make {}: {e}", dir.display())));
    let file = dir.join("index.html");
    fs::write(&file, body).unwrap_or_else(|e| fail(&format!("cannot write {}: {e}", file.display())));
}

/// Pulls a built asset's name out of what Trunk wrote, so the hash is never
/// guessed at.
fn asset(html: &str, ending: &str) -> Option<String> {
    html.split(['"', '\''])
        .find(|part| part.starts_with('/') && part.ends_with(ending))
        .map(|part| part.trim_start_matches('/').to_string())
}

/// The three files a crawler or an agent looks for and, until now, was served
/// the website instead of.
fn crawler_files(dist: &Path, site: &str) {
    let mut urls = String::new();
    let mut listed: Vec<String> = Vec::new();
    for page in PAGES.iter() {
        let loc = format!("{site}/{}", canonical_of(page.slug));
        urls.push_str(&format!("  <url><loc>{loc}</loc></url>\n"));
        listed.push(format!("- [{}]({loc}): {}", page.title, page.summary));
    }

    fs::write(
        dist.join("sitemap.xml"),
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
        ),
    )
    .ok();

    fs::write(
        dist.join("robots.txt"),
        format!("User-agent: *\nAllow: /\n\nSitemap: {site}/sitemap.xml\n"),
    )
    .ok();

    // https://llmstxt.org — a plain summary for anything reading the site
    // rather than looking at it.
    fs::write(
        dist.join("llms.txt"),
        format!(
            "# Xag\n\n\
             > {}\n\n\
             Xag is early and unstable. The compiler is the authority on what the \
             language does; this site is not. Anything here may be out of date, and \
             the repository is where to check.\n\n\
             ## Pages\n\n{}\n\n\
             ## Source\n\n\
             - [Compiler, runtime, engines and oracle]({})\n\
             - [This website]({})\n",
            PAGES[0].summary,
            listed.join("\n"),
            xag_site::content::REPO,
            xag_site::content::SITE_REPO,
        ),
    )
    .ok();
}

fn fail(message: &str) -> ! {
    eprintln!("prerender: {message}");
    std::process::exit(1)
}
