//! The Xag website.
//!
//! Everything shown here was checked against the compiler before it was written
//! down. The programs under `samples/` are files that were run, and the error
//! message is the one `xagc check` actually printed. Nothing on this page
//! describes a language feature that was not exercised first.

mod space;
mod warp;

use xag_site::{content, markup, route, syntax};

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

    // The pieces two rocks come apart into, and the light of the impact. Both
    // are made once and then only moved, because a collision is no time to be
    // building elements.
    let shards = space::shard_shapes()
        .into_iter()
        .map(|points| {
            view! {
                <div class="shard">
                    <svg viewBox="-50 -50 100 100" aria-hidden="true">
                        <polygon points=points />
                    </svg>
                </div>
            }
        })
        .collect_view();

    let flashes = (0..space::FLASH_POOL)
        .map(|_| view! { <div class="flash"></div> })
        .collect_view();

    view! {
        <div class="space" aria-hidden="true">
            <svg class="space-defs" width="0" height="0">
                <defs>
                    <linearGradient id="rock-a" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#3f9e6c" class="lit" />
                        <stop offset="100%" stop-color="#08211a" class="dark" />
                    </linearGradient>
                    <linearGradient id="rock-b" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#54c98c" class="lit" />
                        <stop offset="100%" stop-color="#0b2b21" class="dark" />
                    </linearGradient>
                    <linearGradient id="rock-c" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#2f8f63" class="lit" />
                        <stop offset="100%" stop-color="#071c16" class="dark" />
                    </linearGradient>
                </defs>
            </svg>
            <div class="stars">{stars}</div>
            <div class="rocks">{rocks}</div>
            <div class="shards">{shards}</div>
            <div class="flashes">{flashes}</div>
            <div class="scrim"></div>
        </div>
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

/// Scrolls to a section once the page holding it is in the document.
///
/// Two cases, and they need different things. A card pointing at the page the
/// reader is already on finds its section straight away, because it never left.
/// A card pointing at another page does not: setting the signal does not put
/// the new view in the document in the same breath — measured, it is still
/// absent on the next line and present one task later — so the timeout is what
/// does the work there.
///
/// It deliberately does not wait for an animation frame. A hidden or throttled
/// page is handed no frames at all, and a jump that quietly does nothing is
/// worse than one that happens a beat late.
fn go_to(anchor: &'static str) {
    if scroll_to(anchor) {
        return;
    }

    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Some(win) = web_sys::window() else { return };
    let again = Closure::once_into_js(move || {
        scroll_to(anchor);
    });
    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
        again.as_ref().unchecked_ref(),
        0,
    );
}

/// Says whether it found the thing it was asked to scroll to.
fn scroll_to(anchor: &str) -> bool {
    let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(anchor))
    else {
        return false;
    };
    el.scroll_into_view();
    true
}

/// The warp, and what the name turns out to stand for.
///
/// It is always in the document, hidden, rather than being put there when it is
/// wanted: Leptos does not add a view in the same breath as the signal that
/// asks for it, and `warp::open` has to find its elements straight away.
#[component]
fn WarpOverlay() -> impl IntoView {
    let streaks = (0..warp::STREAKS)
        .map(|_| view! { <i class="streak"></i> })
        .collect_view();

    view! {
        <div class="warp" on:click=move |_| warp::close()>
            <div class="warp-field" aria-hidden="true">{streaks}</div>
            <p class="warp-reveal">
                <span class="phrase">
                    "e"<b>"X"</b>"cellent "<b>"A"</b>"lien lan"<b>"G"</b>"uage"
                </span>
            </p>
            <span class="warp-dismiss">"click anywhere, or press escape"</span>
        </div>
    }
}

/// What the address bar currently says.
fn place_now() -> (Option<route::Theme>, Page) {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".into());
    let (theme, slug) = route::split(&path);
    (theme, Page::from_slug(&slug))
}

/// Writes the address. A page is somewhere you went, so it goes in the
/// history and Back undoes it; a theme is how you are reading, so it replaces
/// what is there rather than filling the history with palette changes.
fn write_address(theme: route::Theme, page: Page, anchor: Option<&str>, went: bool) {
    let Some(history) = web_sys::window().and_then(|w| w.history().ok()) else {
        return;
    };
    let url = route::build(theme, page.slug(), anchor);
    let nothing = wasm_bindgen::JsValue::NULL;
    let _ = if went {
        history.push_state_with_url(&nothing, "", Some(&url))
    } else {
        history.replace_state_with_url(&nothing, "", Some(&url))
    };
}

/// Back and Forward have to work, or the address bar is decoration.
fn follow_history(set_page: WriteSignal<Page>, set_theme: WriteSignal<route::Theme>) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Some(win) = web_sys::window() else { return };

    let on_pop = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
        let (theme, page) = place_now();
        if let Some(theme) = theme {
            set_theme.set(theme);
        }
        set_page.set(page);
    }) as Box<dyn FnMut(web_sys::PopStateEvent)>);

    let _ = win.add_event_listener_with_callback("popstate", on_pop.as_ref().unchecked_ref());
    on_pop.forget();
}

/// Where the reader's choice of theme is kept between visits.
const THEME_KEY: &str = "xag.theme";

/// What the reader chose last time, if they have been here before.
fn stored_theme() -> Option<route::Theme> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|store| store.get_item(THEME_KEY).ok().flatten())
        .and_then(|v| route::Theme::from_slug(&v))
}

/// Which theme a first visit opens in, which is the domain's business.
fn theme_for_host() -> route::Theme {
    let host = web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_default();
    route::default_theme(&host)
}

/// Puts the theme on the document, remembers it, and starts or stops the sky.
///
/// Solarized Dark is a palette somebody else settled, and the point of asking
/// for it is to get it rather than to get this site wearing it. So the sky
/// stops, the wider gamut is switched off in the stylesheet, and what is left
/// is sixteen colours on a flat ground.
fn apply_theme(theme: route::Theme) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        match theme.attribute() {
            Some(name) => {
                let _ = root.set_attribute("data-theme", name);
            }
            None => {
                let _ = root.remove_attribute("data-theme");
            }
        }
    }

    if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = store.set_item(THEME_KEY, theme.slug());
    }

    if theme.has_sky() {
        space::resume();
    } else {
        space::pause();
    }
}

#[component]
fn ThemePicker(theme: ReadSignal<route::Theme>, set_theme: WriteSignal<route::Theme>) -> impl IntoView {
    let options = route::THEMES
        .iter()
        .map(|&t| {
            view! {
                <option value=t.slug() selected=move || theme.get() == t>
                    {t.label()}
                </option>
            }
        })
        .collect_view();

    view! {
        <div class="theme">
            <label for="theme-pick">"Theme"</label>
            <select
                id="theme-pick"
                on:change=move |ev| {
                    if let Some(picked) = route::Theme::from_slug(&event_target_value(&ev)) {
                        set_theme.set(picked);
                    }
                }
            >
                {options}
            </select>
        </div>
    }
}

/// The parts of the site, in the order they sit in the dock.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Download,
    Docs,
    Philosophy,
    Credits,
}

const PAGES: [Page; 5] = [
    Page::Home,
    Page::Download,
    Page::Docs,
    Page::Philosophy,
    Page::Credits,
];

impl Page {
    fn label(self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Download => "Download",
            Page::Docs => "Docs",
            Page::Philosophy => "Design Philosophy",
            Page::Credits => "Credits, License, Source",
        }
    }

    /// What this page is called in an address. The two that lead nowhere have
    /// no slug, because an address for a page that does not exist is a promise
    /// the site cannot keep.
    fn slug(self) -> &'static str {
        match self {
            Page::Home => "",
            Page::Philosophy => "philosophy",
            Page::Credits => "credits",
            Page::Download | Page::Docs => "",
        }
    }

    /// Anything unrecognised is the front page. Somebody who typed a wrong
    /// address should land somewhere rather than nowhere.
    fn from_slug(slug: &str) -> Page {
        match slug {
            "philosophy" => Page::Philosophy,
            "credits" => Page::Credits,
            _ => Page::Home,
        }
    }

    /// Two of these lead nowhere yet, and say so rather than opening on an
    /// apology. There is nothing to download — Xag cannot be installed, only
    /// built — and the documentation is the compiler's README for now.
    fn works(self) -> bool {
        !matches!(self, Page::Download | Page::Docs)
    }

    fn why_not(self) -> &'static str {
        match self {
            Page::Download => "Nothing to download yet — Xag is built from source, not installed",
            Page::Docs => "Not written yet",
            _ => "",
        }
    }
}

#[component]
fn Dock(
    page: ReadSignal<Page>,
    set_page: WriteSignal<Page>,
    theme: ReadSignal<route::Theme>,
) -> impl IntoView {
    let items = PAGES
        .iter()
        .map(|&p| {
            if p.works() {
                view! {
                    <button
                        class="dock-item"
                        class:here=move || page.get() == p
                        aria-current=move || if page.get() == p { "page" } else { "false" }
                        on:click=move |_| {
                            set_page.set(p);
                            write_address(theme.get_untracked(), p, None, true);
                            // A new page starts at its top, not wherever the
                            // last one had been scrolled to.
                            if let Some(win) = web_sys::window() {
                                win.scroll_to_with_x_and_y(0.0, 0.0);
                            }
                        }
                    >
                        {p.label()}
                    </button>
                }
                .into_any()
            } else {
                view! {
                    <span class="dock-item waiting" aria-disabled="true" title=p.why_not()>
                        {p.label()}
                    </span>
                }
                .into_any()
            }
        })
        .collect_view();

    view! { <nav class="dock" aria-label="Sections">{items}</nav> }
}

/// One block of a page.
///
/// Prose goes in as HTML from the shared markup, so what the app shows and what
/// was written into `dist` at build time are the same words rendered the same
/// way — there is no second copy to drift.
#[component]
fn BlockView(block: &'static content::Block) -> impl IntoView {
    use content::Block;
    match block {
        Block::Para(text) => view! { <p inner_html=markup::to_html(text)></p> }.into_any(),
        Block::Claim(text) => {
            view! { <p class="claim" inner_html=markup::to_html(text)></p> }.into_any()
        }
        Block::Warning(text) => {
            view! { <p class="warning" inner_html=markup::to_html(text)></p> }.into_any()
        }
        Block::Code(src) => view! { <Code src=src.to_string() /> }.into_any(),
        Block::Shell(src) => view! { <pre class="shell"><code>{*src}</code></pre> }.into_any(),
        Block::Diagnostic(text) => {
            view! { <pre class="diagnostic"><code>{*text}</code></pre> }.into_any()
        }
        Block::Sample { file, src, output } => {
            view! { <Sample file=file src=src output=output /> }.into_any()
        }
        Block::Questions => view! {
            {content::QUESTIONS.iter().map(|q| view! {
                <h3 id=q.id>{q.asked}</h3>
                <p class="claim" inner_html=markup::to_html(q.short)></p>
                {q.answer.iter().map(|para| view! {
                    <p inner_html=markup::to_html(para)></p>
                }).collect_view()}
            }).collect_view()}
        }
        .into_any(),
        Block::MarksTable => view! {
            <table class="marks-table">
                <tbody>
                    {content::MARKS.iter().map(|(class, mark, meaning)| view! {
                        <tr>
                            <td><code class=*class>{*mark}</code></td>
                            <td inner_html=markup::to_html(meaning)></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        }
        .into_any(),
        Block::LivePanel => view! { <Marks /> }.into_any(),
        Block::Bullets(items) => view! {
            <ul class="engines">
                {items.iter().map(|(lead, rest)| view! {
                    <li>
                        <strong>{*lead}</strong>
                        " — "
                        <span inner_html=markup::to_html(rest)></span>
                    </li>
                }).collect_view()}
            </ul>
        }
        .into_any(),
        Block::Quote { paragraphs, attribution } => view! {
            <blockquote>
                {paragraphs.iter().map(|p| view! {
                    <p inner_html=markup::to_html(p)></p>
                }).collect_view()}
                <footer>{*attribution}</footer>
            </blockquote>
        }
        .into_any(),
    }
}

#[component]
fn PageView(page: &'static content::Page) -> impl IntoView {
    view! {
        {(!page.lede.is_empty()).then(|| view! {
            <p class="lede" inner_html=markup::to_html(page.lede)></p>
        })}
        {page.sections.iter().map(|section| view! {
            <section id=section.id>
                <h2>{section.heading}</h2>
                {section.blocks.iter().map(|b| view! { <BlockView block=b /> }).collect_view()}
            </section>
        }).collect_view()}
    }
}

#[component]
fn App() -> impl IntoView {
    // The address decides where this starts. If it names a theme that wins;
    // if it does not, the reader's last choice does.
    let (named_theme, opened_at) = place_now();
    // The address wins if it names a theme, then what the reader chose last
    // time, then the domain they came in on.
    let starts_as = named_theme
        .or_else(stored_theme)
        .unwrap_or_else(theme_for_host);

    let (page, set_page) = signal(opened_at);
    let at_home = move || page.get() == Page::Home;

    let (theme, set_theme) = signal(starts_as);
    let alien = move || theme.get() == route::Theme::Alien;

    // Whatever the address said or left out, it says all of it from here on —
    // replacing rather than pushing, so arriving does not leave a step behind
    // it that Back would walk into.
    write_address(starts_as, opened_at, None, false);
    follow_history(set_page, set_theme);

    // Runs on mount as well as on every change, so a reader who chose Solarized
    // last time arrives in it rather than watching it swap over. Closing the
    // warp is part of it: the name does not stand for anything in a palette
    // somebody else designed.
    Effect::new(move |_| {
        let now = theme.get();
        apply_theme(now);
        // The reveal is the alien theme's own; it says nothing in the others.
        if now != route::Theme::Alien {
            warp::close();
        }
        write_address(now, page.get_untracked(), None, false);
    });

    view! {
        <header class="hero" class:compact=move || !at_home()>
            <Space />
            <div class="hero-inner">
                <div class="hero-mark">
                    <h1>
                        {move || if !alien() {
                            view! { <span class="wordmark">"Xag"</span> }.into_any()
                        } else {
                            view! {
                                <button
                                    class="wordmark"
                                    title="What it stands for"
                                    on:click=move |_| warp::open()
                                >
                                    "Xag"
                                </button>
                            }.into_any()
                        }}
                    </h1>
                </div>

                <div class="hero-words">
                    <p class="tagline">
                        {content::TAGLINE.iter().map(|(text, good)| view! {
                            <span class=if *good { "good" } else { "cost" }>{*text}</span>
                        }).collect_view()}
                    </p>
                    <p class="warning">
                        "Under development, don't treat this website's info as "
                        <a href=REPO>"truth"</a>"."
                    </p>
                    <a class="cta" href="#building">"Build from source"</a>
                </div>

                {move || at_home().then(|| view! {
                    <nav class="cards" aria-label="Questions">
                        {content::QUESTIONS.iter().map(|q| view! {
                            <button
                                class="card asks"
                                on:click=move |_| {
                                    set_page.set(Page::Home);
                                    write_address(theme.get_untracked(), Page::Home, Some(q.id), true);
                                    go_to(q.id);
                                }
                            >
                                <span class="card-icon" aria-hidden="true">"?"</span>
                                <span class="card-label">{q.asked}</span>
                                <svg class="card-arrow" viewBox="0 0 24 24" aria-hidden="true">
                                    <path d="M4 12h15M13 6l6 6-6 6" />
                                </svg>
                            </button>
                        }).collect_view()}
                    </nav>
                })}
            </div>
        </header>

        <ThemePicker theme=theme set_theme=set_theme />
        <WarpOverlay />

        <p class="vertical-note">
            "Everything on this website is WASM via Rust where possible"
        </p>

        <main>
            // Download and Docs cannot be reached: the dock will not take a
            // click on either of them, so `content::page` never sees them.
            {move || view! { <PageView page=content::page(page.get().slug()) /> }}
        </main>

        <Dock page=page set_page=set_page theme=theme />
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

    // Escape closes the warp, wherever the reader is.
    warp::install();
}
