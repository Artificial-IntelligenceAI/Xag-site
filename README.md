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

Nothing is deployed yet, and no domain is pointed at it.

## Licence

Dual licensed under Apache-2.0 and MIT, matching the compiler.
