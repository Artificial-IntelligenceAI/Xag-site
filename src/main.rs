//! The Xag website.
//!
//! Everything shown here was checked against the compiler before it was written
//! down. The programs under `samples/` are files that were run, and the error
//! message is the one `xagc check` actually printed. Nothing on this page
//! describes a language feature that was not exercised first.

mod space;
mod syntax;

use leptos::prelude::*;
use syntax::tokenize;

const REPO: &str = "https://github.com/Artificial-IntelligenceAI/Xag-lang";

/// Colours a piece of Xag. One span per token, in the order they were read.
fn highlight(src: &str) -> impl IntoView {
    tokenize(src)
        .into_iter()
        .map(|t| view! { <span class=t.kind.class()>{t.text}</span> })
        .collect_view()
}

/// Xag source, coloured by the rule the language already states.
#[component]
fn Code(src: String) -> impl IntoView {
    view! { <pre class="code"><code>{highlight(&src)}</code></pre> }
}

/// A program, and what it printed when it was run.
#[component]
fn Sample(file: &'static str, src: &'static str, output: &'static str) -> impl IntoView {
    view! {
        <figure class="sample">
            <Code src=src.to_string() />
            <figcaption>
                <span class="filename">{file}</span>
                <span class="sep">" — "</span>
                <code class="cmd">"xagc run "{file}</code>
            </figcaption>
            <pre class="output"><code>{output}</code></pre>
        </figure>
    }
}

/// The black, the stars, and the rocks drifting in it.
#[component]
fn Space() -> impl IntoView {
    let stars = space::stars()
        .into_iter()
        .map(|s| {
            let style = format!(
                "left:{:.2}%;top:{:.2}%;width:{:.2}px;height:{:.2}px;opacity:{:.2};\
                 --secs:{:.1}s;--delay:{:.1}s",
                s.left, s.top, s.size, s.size, s.alpha, s.secs, s.delay
            );
            let class = if s.twinkles { "star twinkles" } else { "star" };
            view! { <i class=class style=style></i> }
        })
        .collect_view();

    let rocks = space::rocks()
        .into_iter()
        .map(|r| {
            // Where it starts. Everything after this frame is written by
            // `space::animate`.
            let style = format!(
                "left:{:.2}%;top:{:.2}%;width:{:.0}px;height:{:.0}px",
                r.left, r.top, r.size, r.size
            );
            view! {
                <div class=r.depth.class() style=style>
                    <svg viewBox="-50 -50 100 100" aria-hidden="true">
                        <polygon points=r.points fill=format!("url(#{})", r.gradient) class="body" />
                        <polygon points=r.facet class="facet" />
                    </svg>
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="space" aria-hidden="true">
            <svg class="space-defs" width="0" height="0">
                <defs>
                    <linearGradient id="rock-a" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#3f9e6c" />
                        <stop offset="100%" stop-color="#08211a" />
                    </linearGradient>
                    <linearGradient id="rock-b" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#54c98c" />
                        <stop offset="100%" stop-color="#0b2b21" />
                    </linearGradient>
                    <linearGradient id="rock-c" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#2f8f63" />
                        <stop offset="100%" stop-color="#071c16" />
                    </linearGradient>
                </defs>
            </svg>
            <div class="stars">{stars}</div>
            <div class="rocks">{rocks}</div>
            <div class="scrim"></div>
        </div>
    }
}

/// One of the cards down the right of the hero.
#[component]
fn Card(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <a class="card" href=href>
            <span class="card-icon" aria-hidden="true">{icon}</span>
            <span class="card-label">{label}</span>
            <svg class="card-arrow" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 12h15M13 6l6 6-6 6" />
            </svg>
        </a>
    }
}

#[component]
fn Hero() -> impl IntoView {
    view! {
        <header class="hero">
            <Space />
            <div class="hero-inner">
                <div class="hero-words">
                    <h1>"Xag"</h1>
                    <p class="tagline">
                        "Dot-chained, Safe, Excellent runtime performance, Slow compilation time."
                    </p>
                    <p class="warning">"Early work in progress — nothing here is stable yet."</p>
                    <a class="cta" href="#building">"Build from source"</a>
                </div>
                <nav class="cards" aria-label="Sections">
                    <Card href="#marks" label="Two marks" icon="'*" />
                    <Card href="#ownership" label="Ownership" icon="→" />
                    <Card href="#errors" label="Error messages" icon="^^" />
                    <Card href="#engines" label="Three engines" icon="≡" />
                    <Card href="#building" label="Build from source" icon=">_" />
                </nav>
            </div>
        </header>
    }
}

/// Type in it and the colours follow. It colours; it does not run.
#[component]
fn Marks() -> impl IntoView {
    let (src, set_src) = signal(include_str!("../samples/counting.xag").to_string());

    view! {
        <div class="marks">
            <div class="marks-panes">
                <textarea
                    class="marks-input"
                    spellcheck="false"
                    autocapitalize="off"
                    aria-label="Xag source to colour"
                    prop:value=move || src.get()
                    on:input=move |ev| set_src.set(event_target_value(&ev))
                ></textarea>
                <pre class="code marks-out"><code>
                    {move || highlight(&src.get())}
                </code></pre>
            </div>
            <p class="marks-note">
                "Edit the left side. The right side is coloured by a tokeniser with no
                 parser in it, no lookahead, and no memory of the token before — "
                <em>"it colours; it does not run."</em>
            </p>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Hero />

        // Nothing in it yet, and nothing pretending to be in it either.
        <div class="dock" aria-hidden="true"></div>

        <p class="vertical-note">
            "Everything on this website is WASM via Rust where possible"
        </p>

        <main>
            <p class="lede">
                "A programming language built around high performance, helpful error
                 messages, and Rust-style memory management. There is no "
                <code>"int"</code>" on its own, because there is no size to assume.
                 Programs are compiled ahead of time, either to native code or to a form
                 run by an AOT interpreter."
            </p>

            <section id="marks">
                <h2>"Two marks, and only two"</h2>
                <table class="marks-table">
                    <tbody>
                        <tr>
                            <td><code class="tk-name">"'…'"</code></td>
                            <td>"a "<strong>"name"</strong></td>
                        </tr>
                        <tr>
                            <td><code class="tk-value">"*…*"</code></td>
                            <td>"a "<strong>"written value"</strong></td>
                        </tr>
                        <tr>
                            <td><code class="tk-word">"word"</code></td>
                            <td>"a function, a type, or a segment of a chain"</td>
                        </tr>
                    </tbody>
                </table>
                <p>
                    "A quoted thing is a name wherever you meet it. It never has to be
                     re-read as a value because of where it happens to sit — position is
                     never consulted."
                </p>
                <p>
                    "There is no third mark for text versus number, because the type
                     already answers that: "<code class="tk-value">"*1000*"</code>" is a
                     number under "<code>"int64"</code>" and four characters under "
                    <code>"str"</code>"."
                </p>
                <p class="claim">"There are only two marks and there will only ever be two."</p>
                <Marks />
            </section>

            <section id="chains">
                <h2>"Declarations are chains"</h2>
                <p>
                    "What is unusual about a name lives in the chain that declares it.
                     Every segment but the type has a default, and the default is always
                     the least powerful thing — so a word appears only where there was a
                     choice."
                </p>
                <Code src="var.mut.many.int64 'xs'".to_string() />
                <p>
                    "That one changes, holds several, and holds 64-bit whole numbers.
                     Drop "<code>"mut"</code>" and it does not change. Drop "
                    <code>"many"</code>" and it holds one."
                </p>
            </section>

            <section id="reading">
                <h2>"Reading a program"</h2>
                <p>
                    "Items sit next to each other and are used in order. Nothing is
                     concatenated into a third thing, so there is no "<code>"+"</code>
                    " for text."
                </p>
                <Sample
                    file="counting.xag"
                    src=include_str!("../samples/counting.xag")
                    output="sum to 10 = 55"
                />
            </section>

            <section id="ownership">
                <h2>"What a name owns, and what it lends"</h2>
                <p>
                    "Ownership is Rust's, checked when the program is compiled, with no
                     garbage collector under it. A transfer is spelled at the call site,
                     so a reader never has to work out from a function's signature that a
                     value has left."
                </p>
                <Sample
                    file="borrowing.xag"
                    src=include_str!("../samples/borrowing.xag")
                    output="size: 5\nafter: hello!\nlonger: hello!\nkept: spare"
                />
            </section>

            <section id="errors">
                <h2>"When it has something to say"</h2>
                <p>
                    "Take the program above, hand a value over for good, and then ask for
                     it again. This is what comes back — not a paraphrase of it:"
                </p>
                <pre class="diagnostic"><code>{include_str!("../samples/moved.output.txt")}</code></pre>
                <p>
                    "A diagnostic that points at the wrong thing, or names a rule the
                     program did not break, is a bug of the same kind as miscompiling —
                     the compiler is telling the reader something untrue either way."
                </p>
            </section>

            <section id="engines">
                <h2>"Three engines and an oracle"</h2>
                <p>"There are three ways to run an Xag program, and they are kept apart on purpose."</p>
                <ul class="engines">
                    <li>
                        <strong>"A test interpreter"</strong>
                        " — built to be obviously correct rather than fast. It walks the IR
                         as written and does nothing clever anywhere. It is the one to
                         believe when the engines disagree."
                    </li>
                    <li>
                        <strong>"A fast interpreter"</strong>
                        " — it turns the graph into flat code once and then runs it without
                         looking anything up again. It shares nothing with the test
                         interpreter but the runtime, on purpose: two engines that borrow
                         from each other agree about what they borrowed, and a vote between
                         them proves nothing."
                    </li>
                    <li>
                        <strong>"A native backend"</strong>
                        " — compiled ahead of time through LLVM."
                    </li>
                </ul>
                <p class="claim">
                    "Two engines can say that something is wrong; three can say which."
                </p>
                <p>
                    "A program generator writes random Xag programs, asks every engine what
                     they say, and when they differ reports which one is out of step with
                     the other two."
                </p>
                <p>
                    "Decimal gets a check the oracle cannot give it. Three engines calling
                     one runtime agree about everything inside that runtime, so a mistake
                     in the arithmetic itself is one all three would make together. So the
                     decimal results are checked against Python's "<code>"decimal"</code>
                     " — libmpdec, written by someone else from the same IBM specification,
                     and derived from nothing here. Four hundred thousand cases agree
                     exactly, apart from raising to a power, which the specification itself
                     allows to be out by one in the last place."
                </p>
                <p>
                    "They are also asked of a real decimal floating-point unit. No machine
                     here has one, so the test cross-compiles a program with no operating
                     system under it, hands it to QEMU where a kernel would go, and reads
                     the answers back over the console. Twenty thousand sums, differences,
                     products and quotients agree exactly, cohorts included."
                </p>
            </section>

            <section id="building">
                <h2>"Building it"</h2>
                <p>
                    "The compiler is written in C++20 against LLVM's native C++ API. It
                     needs LLVM 23 or newer, CMake and Ninja."
                </p>
                <pre class="shell"><code>{
"git clone https://github.com/Artificial-IntelligenceAI/Xag-lang.git\n\
cd Xag-lang\n\
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release\n\
ninja -C build\n\
./build/xagc run examples/counting.xag"
                }</code></pre>
                <p>
                    <code>"xagc run"</code>" runs a program on the test interpreter, "
                    <code>"xagc fast"</code>" on the fast one, and "<code>"xagc build"</code>
                    " compiles it ahead of time and writes an executable beside it. "
                    <code>"xagc check"</code>" checks a program without running it."
                </p>
                <p class="warning">
                    "There is no install yet. Building from source is the way to run it."
                </p>
            </section>

            <section id="honest">
                <h2>"Why would anyone ever use Xag?"</h2>
                <blockquote>
                    <p>"Honestly, I don't know. 😂"</p>
                    <p>
                        "The code written is mostly or 100% AI-made. Why? Because, I'm more
                         of a designer, not a C++ stroke-inducing syntax reader 🤣.
                         No offense."
                    </p>
                    <footer>"— the README"</footer>
                </blockquote>
                <p>
                    "Wrong diagnostics, confusing ones, and anything Xag accepts that it
                     should not are worth reporting: "
                    <a href=format!("{REPO}/issues")>{format!("{REPO}/issues")}</a>
                </p>
            </section>
        </main>

        <footer class="site-footer">
            <p>
                <a href=REPO>"The compiler on GitHub"</a>
                <span class="sep">" · "</span>
                "Dual licensed Apache-2.0 and MIT"
            </p>
            <p class="fine">"Copyright 2026 Tankun Sriket"</p>
        </footer>
    }
}

fn main() {
    console_error_panic_hook::set_once();

    // The no-WebAssembly fallback in index.html has done its job by now.
    if let Some(el) = document().get_element_by_id("fallback") {
        el.remove();
    }

    leptos::mount::mount_to_body(App);

    // The rocks are drawn by the markup above and moved by this.
    space::animate();
}
