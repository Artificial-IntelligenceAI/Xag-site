# The Xag website — brief

This repo is the website for Xag, the language whose compiler lives at
`github.com/Artificial-IntelligenceAI/Xag-lang`. It is a separate repo on
purpose: the site can be rebuilt, redeployed and broken without touching the
compiler's history.

## Who decides what

Tankun designs. That is not a formality — it is the standing rule across every
Xag session. **What the site is for, what pages it has, how it is built and where
it is deployed are all his calls, and none of them are settled yet.** Ask him
before building anything. Suggest options with trade-offs; do not pick for him
and do not treat this brief as having decided any of it.

What this brief settles is only the ground truth: what Xag actually is today,
which parts of that are safe to say, and how to check the rest.

## The one design constraint that is already fixed

Tankun's own framing when he asked for this session: *"the website won't be
having anything much yet, so we won't have much stale surface to maintain."*

That is the governing principle. A page that claims something the language does
not do is worse than no page. The language is moving weekly — structs landed the
day this repo was made — so anything enumerating features will be wrong within
the month. Prefer saying less.

## Do not invent syntax

The compiler is the authority on what Xag looks like, and it is right here. Every
code sample on the site should be one you have actually run.

```sh
git clone https://github.com/Artificial-IntelligenceAI/Xag-lang.git
cd Xag-lang
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release && ninja -C build
./build/xagc run examples/grouping.xag
```

A local checkout may already be at `/Users/ts/SafetyBolt language` — ask Tankun
rather than assuming. Read `README.md` and `design/syntax.md` there first; the
`examples/` directory holds programs that are known to run, and they are the
safest source of samples. `xagc check <file>` on a deliberately broken program
gives you a real error message, which is worth showing verbatim rather than
paraphrasing.

## What is settled enough to publish

- **What Xag is.** Compiled ahead of time, to native code or to a form run by an
  AOT interpreter. Rust-style ownership, checked at compile time. A size is
  always written — there is no `int` on its own, because there is no size to
  assume. Error messages that explain themselves.
- **The two marks.** `'name'` is a variable, `*written value*` is a literal, and
  a bare word is a type, a segment of a chain, or a function being called. Every
  declaration marks the name it gives, a function's and a struct's included.
  There are only two marks and there will only ever be two.
- **Dot-chained declarations.** What is unusual about a name lives in the chain
  that declares it: `var.mut.many.int64 'xs'`. The default is always the least
  powerful thing, so a word appears only where there was a choice.
- **The types.** Sized whole numbers, IEEE 754 binary, and IEEE 754 decimal —
  all four families are complete, including a decimal that has been checked
  against an independent implementation and against IBM POWER's hardware unit.
- **Three engines and an oracle.** A test interpreter built to be obviously
  correct, a fast interpreter, and an AOT native backend — plus a program
  generator that writes random programs and refuses to accept an answer the three
  do not agree on. This is the most distinctive thing about the project and it
  reads well to anyone who has shipped a compiler.
- **Building from source.** The README's instructions are current and tested.
- **That it is early.** The README says "nothing here is stable yet" and says who
  wrote the code and why. That honesty is an asset, not something to smooth over.

## What will go stale — leave it off

- **Feature lists.** Modules, generics, growable arrays and nested arrays do not
  exist yet. Structs arrived on 2026-09-06. Anything enumerating what the
  language has will be wrong soon.
- **Install instructions.** Xag installs as of 2026-09-07 — `cmake --install`
  puts a working compiler under a prefix, and `xagc` finds its runtime rather
  than having the path baked in. What there is no such thing as is a *release*:
  no archive, no formula, no version to ask for. Say that, and leave the
  commands to the README, which changes with them.
- **Performance claims.** There are no published numbers and the fast interpreter
  is actively being optimised in another session.
- **The open design questions.** `design/syntax.md` ends with a handful of them
  — five on 2026-09-07, seven when this was written. Do not count them here;
  the number is one more thing to keep true, and the answer is in that file.

### A measured number goes stale too, and looks like it cannot

Everything above is a claim, and a claim that is wrong reads as a mistake. A
benchmark result reads as a fact, which is what makes it the more dangerous
thing to put on a page: nobody re-checks a fact.

On 2026-09-06 a billion-iteration loop was measured at twenty times slower than
the same loop in C. By that afternoon the compiler emitted the machine's own
instruction and the same loop was one-and-a-bit times slower. Both numbers were
correct when taken, four hours apart. A page carrying the first one would have
been wrong by lunchtime, and would have gone on being wrong quietly, because
nothing about it would have looked like it needed revisiting.

So the rule is not that measurements are unreliable. It is that a number is
about the afternoon it was taken on, and a page has no way of saying so. The
compiler repo keeps `bench/run.py` for asking the question again; the site
should keep the answer off the pages and point at the question instead.

The same thing happens to syntax, and faster. On 2026-09-06 declarations began
marking the names they give, so `fn.int64 sum-to` became `fn.int64 'sum-to'`
and every sample on the site was wrong in the same instant — along with the
sentence above describing what a bare word is. Nothing looked broken. This is
why the rule is to run every sample rather than to copy one: a program that is
run says whether it still works, and a program that is pasted only says what
was true when somebody pasted it.

## Voice

Match the README rather than the average language homepage. It is plain, direct,
and does not oversell — it literally asks *"Why would anyone ever use Xag?"* and
answers *"Honestly, I don't know."* That voice is Tankun's and it is better than
anything a landing page usually says. Do not replace it with vendor copy, and do
not write marketing superlatives.

Never state how long something will take. Describe scope and what is unknown
instead. This has come up repeatedly and matters.

## The domains

`xag-lang.com` and `xag-lang.org`, and **only** those. If the site needs a
canonical URL, ask Tankun which of the two is primary and whether they are
registered yet.

## Talking to the compiler session

There is another Claude session working on the Xag compiler itself. You can reach
it with `mcp__ccd_session_mgmt__send_message`; find its id with
`mcp__ccd_session_mgmt__list_sessions` (its title mentions the language). Use it
when you need to know whether something is true of the language today, whether a
claim is safe to publish, or when a sample stops working after a change. It would
rather answer than have the site say something wrong.

## Conventions carried over from the compiler repo

- Commits are authored `Tankun Sriket <tankun.sriket@safetyboltlang.invalid>`,
  with a `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>` trailer.
  Tankun's own commits use his real address; that is deliberate, leave it.
- The site is dual licensed Apache-2.0 and MIT; both files are here. **The
  compiler is not** — it is `Apache-2.0 WITH LLVM-exception` as of 2026-09-07,
  so that using Xag to build something costs no attribution (the runtime is
  linked into every program it builds) while changing Xag itself asks for a
  note saying what changed. The site's licence page states the compiler's
  terms, so it goes stale the way a code sample does.
- Commit messages say what changed and why, in sentences, without a type prefix.
