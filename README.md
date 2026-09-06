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

## Checking the playground against the compiler

The playground on `/playground` runs Xag in the browser, on a small interpreter
written for this site (`src/play.rs`). It is a second implementation of a
language, which is a thing that drifts, and a page teaching a syntax the
compiler does not accept is worse than no page.

```sh
XAGC=/path/to/xagc cargo run --features prerender --bin verify-play
```

That runs every program the playground ships, and a handful it should refuse,
through both the interpreter and a real `xagc`, and exits non-zero if they
disagree. It is not in CI, because CI has no compiler to hand; somebody has to
run it after either side changes.

A disagreement is not a bug to be patched into agreement. It means the
playground is telling readers something untrue, and the fixes are to correct it
or to stop claiming that much — `src/play.rs` carries a list of what it does not
know, and that list is allowed to grow.

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

The two domains are the same site and differ only in where they start: `.org`
opens in silver, `.com` in alien. It is the last word in the order, not the
first — an address that names a theme wins, and so does a choice the reader
made on a previous visit.

`xag-lang.com` and `xag-lang.org` are both registered. The `.com` is canonical
and is what `index.html` names; pointing the `.org` at it is a redirect for
whatever ends up serving the site. Nothing is deployed yet.

## Licence

Dual licensed under Apache-2.0 and MIT, matching the compiler.
