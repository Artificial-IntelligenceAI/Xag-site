//! What happens when somebody clicks the name.
//!
//! The page falls away, the stars stretch into lines and come at you, and the
//! name turns out to have been standing for something all along.
//!
//! The motion is worked out here rather than in a stylesheet for the same
//! reason the rocks are: a streak accelerates, is drawn longer the faster it
//! goes, and starts again from the middle when it leaves — which is a state
//! being carried forward rather than two poses to interpolate between.

use std::f64::consts::TAU;

/// Enough to fill the view without asking much of anything.
pub const STREAKS: usize = 150;

/// When the words begin to arrive, in seconds.
const REVEAL_AT: f64 = 0.85;

/// How long they take to arrive.
const REVEAL_OVER: f64 = 0.9;

/// The same xorshift as everywhere else here.
struct Rng(u32);

impl Rng {
    fn bits(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn unit(&mut self) -> f64 {
        (self.bits() >> 8) as f64 / (1u32 << 24) as f64
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }
}

pub struct Streak {
    /// Which way out from the middle.
    pub angle: f64,
    /// How far along that bearing.
    pub dist: f64,
    /// Pixels a second before the acceleration is applied.
    speed: f64,
    /// How long it is drawn, which is how fast it is going.
    pub len: f64,
    pub bright: f64,
}

pub struct Warp {
    pub streaks: Vec<Streak>,
    pub elapsed: f64,
    rng: Rng,
}

impl Default for Warp {
    fn default() -> Self {
        Self::new()
    }
}

impl Warp {
    pub fn new() -> Self {
        let mut rng = Rng(0x57A2_1CED);
        let streaks = (0..STREAKS)
            .map(|_| Streak {
                angle: rng.range(0.0, TAU),
                // Spread through the depth of it at the start, so it does not
                // begin as a ring leaving all at once.
                dist: rng.range(0.0, 620.0),
                speed: rng.range(90.0, 420.0),
                len: 2.0,
                bright: rng.range(0.35, 1.0),
            })
            .collect();
        Warp { streaks, elapsed: 0.0, rng }
    }

    /// Back to nothing, ready to be opened again.
    pub fn restart(&mut self) {
        *self = Warp::new();
    }

    /// How fast everything is going by now. It keeps building: the whole
    /// feeling of the thing is that it has not finished accelerating.
    pub fn boost(&self) -> f64 {
        1.0 + self.elapsed * self.elapsed * 2.6
    }

    /// Nothing, then the words.
    pub fn reveal(&self) -> f64 {
        ((self.elapsed - REVEAL_AT) / REVEAL_OVER).clamp(0.0, 1.0)
    }

    pub fn step(&mut self, secs: f64, width: f64, height: f64) {
        self.elapsed += secs;
        let boost = self.boost();
        // The far corner: a streak is gone once it is past this.
        let reach = (width * width + height * height).sqrt() / 2.0;

        for i in 0..self.streaks.len() {
            let speed = self.streaks[i].speed * boost;
            self.streaks[i].dist += speed * secs;
            self.streaks[i].len = (speed * 0.055).clamp(2.0, 260.0);

            if self.streaks[i].dist - self.streaks[i].len > reach {
                // Away again from the middle, on a new bearing.
                let angle = self.rng.range(0.0, TAU);
                let speed = self.rng.range(90.0, 420.0);
                let bright = self.rng.range(0.35, 1.0);
                let s = &mut self.streaks[i];
                s.angle = angle;
                s.dist = 0.0;
                s.speed = speed;
                s.bright = bright;
            }
        }
    }
}

// ─────────────────────────── the browser half ───────────────────────────

#[cfg(target_arch = "wasm32")]
mod live {
    use super::*;
    use std::cell::{Cell, RefCell};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    thread_local! {
        static WARP: RefCell<Warp> = RefCell::new(Warp::new());
        static OPEN: Cell<bool> = const { Cell::new(false) };
        static LOOPING: Cell<bool> = const { Cell::new(false) };
    }

    pub fn is_open() -> bool {
        OPEN.with(|o| o.get())
    }

    pub fn open() {
        if is_open() {
            return;
        }
        WARP.with(|w| w.borrow_mut().restart());
        OPEN.with(|o| o.set(true));

        if let Some(el) = overlay() {
            let _ = el.style().set_property("display", "block");
            // On its own line so the fade is a transition rather than a jump.
            let _ = el.class_list().add_1("running");
        }

        run();

        // If the loop could not start there is nothing to fade the words in,
        // and the overlay would be a black screen that says nothing. Better a
        // reveal with no warp behind it than a warp with no reveal.
        if !LOOPING.with(|l| l.get()) {
            if let Some(reveal) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".warp-reveal").ok().flatten())
                .and_then(|e| e.dyn_into::<HtmlElement>().ok())
            {
                let _ = reveal.style().set_property("opacity", "1");
                let _ = reveal
                    .style()
                    .set_property("transform", "translate(-50%, -50%) scale(1)");
            }
        }
    }

    pub fn close() {
        if !is_open() {
            return;
        }
        OPEN.with(|o| o.set(false));
        if let Some(el) = overlay() {
            let _ = el.class_list().remove_1("running");
            let _ = el.style().set_property("display", "none");
        }
    }

    fn overlay() -> Option<HtmlElement> {
        web_sys::window()?
            .document()?
            .query_selector(".warp")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    }

    fn run() {
        if LOOPING.with(|l| l.get()) {
            return;
        }

        let Some(win) = web_sys::window() else { return };
        let Some(doc) = win.document() else { return };

        let Ok(nodes) = doc.query_selector_all(".streak") else { return };
        if nodes.length() as usize != STREAKS {
            return;
        }
        let mut els: Vec<HtmlElement> = Vec::with_capacity(STREAKS);
        for i in 0..STREAKS {
            let Some(el) = nodes
                .get(i as u32)
                .and_then(|n| n.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            els.push(el);
        }
        let reveal = doc
            .query_selector(".warp-reveal")
            .ok()
            .flatten()
            .and_then(|e| e.dyn_into::<HtmlElement>().ok());

        LOOPING.with(|l| l.set(true));

        let frame: std::rc::Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> =
            std::rc::Rc::new(RefCell::new(None));
        let next = frame.clone();
        let last = std::rc::Rc::new(Cell::new(f64::NAN));

        *next.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
            let Some(win) = web_sys::window() else { return };

            if !is_open() {
                LOOPING.with(|l| l.set(false));
                return;
            }

            let previous = last.get();
            last.set(now);
            let secs = if previous.is_nan() {
                0.0
            } else {
                ((now - previous) / 1000.0).clamp(0.0, 0.05)
            };

            let width = win
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1280.0);
            let height = win
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);

            WARP.with(|cell| {
                let mut warp = cell.borrow_mut();
                warp.step(secs, width, height);

                for (el, streak) in els.iter().zip(warp.streaks.iter()) {
                    let _ = el.style().set_property(
                        "transform",
                        &format!(
                            "rotate({:.4}rad) translateX({:.1}px)",
                            streak.angle, streak.dist
                        ),
                    );
                    let _ = el.style().set_property("width", &format!("{:.1}px", streak.len));
                    let _ = el
                        .style()
                        .set_property("opacity", &format!("{:.3}", streak.bright));
                }

                if let Some(reveal) = reveal.as_ref() {
                    let shown = warp.reveal();
                    let _ = reveal.style().set_property("opacity", &format!("{:.3}", shown));
                    // Arrives from a way off rather than simply appearing.
                    let _ = reveal.style().set_property(
                        "transform",
                        &format!("translate(-50%, -50%) scale({:.3})", 0.86 + 0.14 * shown),
                    );
                }
            });

            if let Some(cb) = frame.borrow().as_ref() {
                let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
            }
        }) as Box<dyn FnMut(f64)>));

        let first = next.borrow();
        if let Some(cb) = first.as_ref() {
            let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }

    /// Escape closes it, as it closes everything else that covers a page.
    pub fn install() {
        let Some(win) = web_sys::window() else { return };
        let Some(doc) = win.document() else { return };

        let on_key = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Escape" {
                close();
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        let _ = doc.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
        on_key.forget();
    }
}

#[cfg(target_arch = "wasm32")]
pub use live::{close, install, is_open, open};

#[cfg(not(target_arch = "wasm32"))]
pub fn open() {}
#[cfg(not(target_arch = "wasm32"))]
pub fn close() {}
#[cfg(not(target_arch = "wasm32"))]
pub fn install() {}
#[cfg(not(target_arch = "wasm32"))]
pub fn is_open() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f64 = 1440.0;
    const H: f64 = 900.0;

    #[test]
    fn everything_leaves_the_middle() {
        let mut warp = Warp::new();
        let before: Vec<f64> = warp.streaks.iter().map(|s| s.dist).collect();
        warp.step(0.05, W, H);
        for (s, was) in warp.streaks.iter().zip(before) {
            assert!(s.dist > was, "a streak went nowhere");
        }
    }

    /// It has to keep building. A warp at a steady speed is a screensaver.
    #[test]
    fn it_does_not_stop_accelerating() {
        let mut warp = Warp::new();
        let mut seen = warp.boost();
        for _ in 0..40 {
            warp.step(0.05, W, H);
            let now = warp.boost();
            assert!(now > seen, "it stopped speeding up");
            seen = now;
        }
    }

    /// The faster it goes the longer it is drawn, which is the whole look.
    #[test]
    fn speed_is_drawn_as_length() {
        let mut warp = Warp::new();
        warp.step(0.05, W, H);
        let early: f64 = warp.streaks.iter().map(|s| s.len).sum();
        for _ in 0..30 {
            warp.step(0.05, W, H);
        }
        let later: f64 = warp.streaks.iter().map(|s| s.len).sum();
        assert!(later > early * 1.5, "{early} to {later} is not a stretch");
    }

    /// They come round again rather than piling up against the edge.
    #[test]
    fn what_leaves_comes_back_from_the_middle() {
        let mut warp = Warp::new();
        let reach = (W * W + H * H).sqrt() / 2.0;
        for _ in 0..200 {
            warp.step(0.02, W, H);
        }
        assert!(
            warp.streaks.iter().all(|s| s.dist - s.len <= reach + 1.0),
            "something is stuck outside the frame"
        );
        assert!(
            warp.streaks.iter().any(|s| s.dist < reach * 0.5),
            "nothing came back to the middle"
        );
    }

    #[test]
    fn the_words_wait_and_then_arrive() {
        let mut warp = Warp::new();
        warp.step(0.1, W, H);
        assert_eq!(warp.reveal(), 0.0, "they turned up straight away");

        for _ in 0..100 {
            warp.step(0.05, W, H);
        }
        assert_eq!(warp.reveal(), 1.0, "they never finished arriving");
    }

    #[test]
    fn opening_it_again_starts_from_the_beginning() {
        let mut warp = Warp::new();
        for _ in 0..60 {
            warp.step(0.05, W, H);
        }
        assert!(warp.elapsed > 0.0);
        warp.restart();
        assert_eq!(warp.elapsed, 0.0);
        assert_eq!(warp.reveal(), 0.0);
    }
}
