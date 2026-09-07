# Xag website

The website for [Xag](https://github.com/Artificial-IntelligenceAI/Xag-lang), a
programming language built around high performance, helpful error messages, and
Rust-style memory management.

`SITE-BRIEF.md` says what Xag actually is today, which parts of that are safe to
publish, and which would go stale.

## What it is

One page, rendered by WebAssembly. The page is a Rust app built with Leptos and
served as static files, so there is nothing to run on the far side of it.

`index.html` carries the language's description and a `<noscript>` note in plain
HTML, and the app removes that block once it has mounted — so a reader without
WebAssembly is told the true thing rather than nothing at all.

### Nothing on the page went unchecked

The programs under `samples/` are files that were run against the compiler
before they were written down, and `samples/moved.output.txt` is the message
`xagc check` actually printed for `samples/moved.xag` — not a paraphrase of it.

```sh
xagc run samples/counting.xag        # sum to 10 = 55
xagc check samples/moved.xag         # the diagnostic shown on the page
```

### The colours are the language's grammar

There are two marks in Xag, so the site has exactly two accent colours: one for
a name, one for a written value. Everything else is neutral, because a third
accent would contradict the thing the page is saying.

`src/syntax.rs` colours the code. It is a tokeniser and nothing more — it does
not run Xag and is not a second implementation of it. What makes it short is the
language rather than any cleverness: a quoted thing is a name wherever it is met
and a starred thing is a written value wherever it is met, so it has no parser,
no lookahead, and no memory of the token before it. Its tests check that, and
that colouring a program never loses it.

## Where the benchmark went

It lives in the compiler repo now, at `bench/run.py`, because it measures the
compiler and cannot run without it. Nothing it produces belongs on the site —
see `SITE-BRIEF.md` on why a measured number goes stale in a way that does not
look like going stale.

## Deploying it

Cloudflare Pages, serving `dist/` as static files. Both domains point at the
same project — the app reads which one it is on.

A push to `main` deploys it: `.github/workflows/deploy.yml` runs the tests,
builds, checks what it is about to ship, and hands `dist/` to Cloudflare. It
needs two repository secrets — `CLOUDFLARE_API_TOKEN`, scoped to Cloudflare
Pages: Edit, and `CLOUDFLARE_ACCOUNT_ID`.

It uses no third-party actions. Everything but checking out and caching is a
command that can be read in the file and run by hand, which is the same reason
the compiler's oracle has no dependencies.

By hand, when that is wanted:

```sh
trunk build --release          # with `trunk serve` NOT running
cargo run --features prerender --bin prerender -- dist https://xag-lang.com
npx wrangler pages deploy dist --project-name=xag-site
```

The middle step is what makes the site readable without WebAssembly. Trunk
writes one `index.html` for every address; the generator replaces it with a real
document per route — that page's prose, its own title, and the palette the
address asked for — and writes `robots.txt`, `sitemap.xml`, `llms.txt` and
`llms-full.txt` beside them — the last being the whole site as one plain
document, for anything reading rather than looking. The app then mounts over whatever was served, so a browser gets
the words first and the sky a moment later.

Both readings come from `src/content.rs`. There is one copy of every sentence,
because two would disagree eventually.

**Stop `trunk serve` before building for deploy.** The dev server writes to the
same `dist/`, and what it writes carries an autoreload client that opens a
WebSocket to an address only the dev server has. Deployed, that is a script on
every page reconnecting forever to nothing. `trunk build --release` on its own
does not include it; the check is that `dist/index.html` contains no
`__TRUNK_ADDRESS__`.

`_redirects` makes every path serve `index.html`, without which the routing has
nothing to read. `_headers` keeps the page itself uncached — every path is the
same HTML, and a cached one strands a reader on an old build — while the hashed
assets are kept for a year, since a changed file gets a changed name.

## Building it

Needs Rust with the `wasm32-unknown-unknown` target, and
[Trunk](https://trunkrs.dev).

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

```sh
trunk serve              # http://127.0.0.1:8080
trunk build --release    # static files into dist/
cargo test               # the tokeniser's tests
```

## Addresses

Every place on the site has one, and it carries the theme as well as the page:

```
/alien                  the front page
/lite/philosophy        Design Philosophy, read in the lite theme
/alien/credits#license  and a section within it
```

`alien` is the site's own look. `silver` is the same place in a colder light:
the sky stays and the rocks read as ore rather than moss. `lite` and `lite2`
are Solarized Dark and Solarized Light, both with the sky, the simulation and
the wider gamut switched off — lite is the honest name for what they are.

Silver is a theme and not the identity. Ag is silver and XAG is what a troy
ounce of it trades under, which is a coincidence the name can wear without
being about it. An address that
names no theme — `/philosophy` — uses whichever was chosen last and then
rewrites itself to say so, so what is in the bar is always the whole
instruction.

Solarized Light is Ethan Schoonover's palette as specified, which means its
contrast is his and not WCAG's: body text lands at 4.13:1 against base3 where
AA asks for 4.5:1, and the accents are lower still. Taking body text down to
base01 would pass and would stay inside the palette, but it would no longer be
the thing it is named after.

Any path serves `index.html` and the app reads it from there, which is what
`_redirects` is for. A host that does not honour it will 404 on everything but
the root.

A theme comes from the address if it names one, then from what the reader chose
last time, then alien. Nothing is decided by which host was asked, because only
one of them answers.

[xag-lang.com](https://xag-lang.com) is the site. `www` on it answers too, and
[xag-lang.org](https://xag-lang.org) with its own `www` redirects there — a
Cloudflare rule on that zone, ahead of Pages, so nothing on the `.org` side ever
reaches the app. It was serving the same documents until 2026-09-07, opening in
silver rather than alien; redirecting is one address to keep true instead of
four, and silver is still at `/silver` for anyone who wants it.

## Credits

**Tankun Sriket designs Xag. Claude writes the code.** The language, the syntax
and what the project is for are his; the compiler, the runtime, the engines, the
oracle and this website are written by Anthropic's Claude under his direction.
That is the arrangement as it stands, and this line changes when it does.

What this repository is made of:

| | |
| --- | --- |
| Rust | the whole site — routing, markup, the sky, the prose as data |
| CSS | written by hand, one file |
| HTML | one file, the Trunk template |
| JavaScript | a few lines in that file, which start the WebAssembly and nothing else |
| Xag | five sample programs, run against the real compiler before they go on a page |
| TOML, YAML, Markdown | build, deploy and this |

What it is built with: [Leptos](https://leptos.dev) draws the pages,
[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) and `web-sys` reach the
browser, [Trunk](https://trunkrs.dev) builds it, and
[Binaryen](https://github.com/WebAssembly/binaryen)'s `wasm-opt` shrinks what
ships. `console_error_panic_hook` puts a Rust panic in the browser console.
Cloudflare Pages serves it.

The two light themes are **Solarized**, by Ethan Schoonover, used as specified
rather than adjusted to taste. No typeface is bundled or fetched — the pages ask
for whatever the machine already has.

The compiler's own credits, including LLVM, the Unicode Character Database,
libmpdec and QEMU, are in [its
README](https://github.com/Artificial-IntelligenceAI/Xag-lang#credits) and on
[the credits page](https://xag-lang.com/alien/credits).

## Licence

`LICENSE` governs, and it splits the repository in two. The code — Rust, HTML,
CSS, build scripts, and the comments in them — is Apache-2.0 with one added
condition: anything built on it must show the `NOTICE` line **where its own
users can see it**, not only in the source. The prose the site displays is not
licensed at all; copyright on it is reserved. Take the machinery, write your own
words.

That added condition means the repository as a whole is not open source by the
OSI definition, which `LICENSE` says out loud rather than leaving to be
inferred. Say "the site's code is open" and not "the site is open source".

MIT is gone. A dual grant lets the taker pick, and MIT asks only that a
copyright line survive somewhere in the source — which is not the credit that
was wanted.

**The compiler is licensed differently**: `Apache-2.0 WITH LLVM-exception` as of
2026-09-07, so a program built with Xag — which has the runtime linked into it —
owes no attribution, while a modified compiler does. The licence page in
`src/content.rs` states the compiler's terms and the site's, and has to keep
matching both.
