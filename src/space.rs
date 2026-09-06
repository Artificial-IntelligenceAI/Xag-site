//! The black behind the page, and the rocks drifting in it.
//!
//! Everything here is worked out once, from a fixed seed, so the sky is the
//! same sky on every load and in every browser. There is no dependency under
//! it: the random numbers are six lines, which is fewer lines than asking a
//! crate for them would cost.

use std::f32::consts::TAU;

/// xorshift32. Not for anything that matters — only for deciding where to put
/// a rock.
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

    /// A number from 0 up to but not including 1.
    fn unit(&mut self) -> f32 {
        (self.bits() >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.unit() * (hi - lo)
    }

    fn below(&mut self, n: u32) -> u32 {
        self.bits() % n
    }
}

/// How far away a rock is. Distance is drawn rather than stated: the far ones
/// are smaller, dimmer, blurred, and drift less.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    Far,
    Middle,
    Near,
}

impl Depth {
    pub fn class(self) -> &'static str {
        match self {
            Depth::Far => "rock far",
            Depth::Middle => "rock middle",
            Depth::Near => "rock near",
        }
    }
}

pub struct Rock {
    pub left: f32,
    pub top: f32,
    pub size: f32,
    pub points: String,
    pub facet: String,
    pub gradient: &'static str,
    pub depth: Depth,
    /// Pixels a second, each on its own heading. Nothing here oscillates: a
    /// rock leaves the way it was going and comes back around the far side.
    pub vx: f32,
    pub vy: f32,
    /// Degrees a second.
    pub spin_rate: f32,
}

pub struct Star {
    pub left: f32,
    pub top: f32,
    pub size: f32,
    pub alpha: f32,
    pub twinkles: bool,
    pub secs: f32,
    pub delay: f32,
}

/// An irregular closed outline, drawn once and then only ever moved.
///
/// The vertices are evenly spaced around a circle and then pushed about, which
/// is enough to stop a rock looking like a polygon and not enough to let one
/// fold in on itself.
fn outline(rng: &mut Rng, lo: f32, hi: f32) -> String {
    let corners = 7 + rng.below(5) as usize;
    let mut pts = String::new();
    for i in 0..corners {
        let angle = (i as f32 / corners as f32) * TAU + rng.range(-0.13, 0.13);
        let radius = rng.range(lo, hi);
        if i > 0 {
            pts.push(' ');
        }
        pts.push_str(&format!("{:.1},{:.1}", angle.cos() * radius, angle.sin() * radius));
    }
    pts
}

/// The lit face of a rock: a smaller outline, pushed towards one side, so the
/// light appears to be coming from somewhere.
fn facet(rng: &mut Rng) -> String {
    let corners = 5 + rng.below(3) as usize;
    let ox = rng.range(-9.0, -3.0);
    let oy = rng.range(-9.0, -3.0);
    let mut pts = String::new();
    for i in 0..corners {
        let angle = (i as f32 / corners as f32) * TAU + rng.range(-0.2, 0.2);
        let radius = rng.range(11.0, 21.0);
        if i > 0 {
            pts.push(' ');
        }
        pts.push_str(&format!(
            "{:.1},{:.1}",
            angle.cos() * radius + ox,
            angle.sin() * radius + oy
        ));
    }
    pts
}

pub fn rocks() -> Vec<Rock> {
    let mut rng = Rng(0x5A67_11E1);
    let gradients = ["rock-a", "rock-b", "rock-c"];
    let mut out = Vec::new();

    for i in 0..22u32 {
        let depth = match i % 3 {
            0 => Depth::Far,
            1 => Depth::Middle,
            _ => Depth::Near,
        };

        let size = match depth {
            Depth::Far => rng.range(22.0, 46.0),
            Depth::Middle => rng.range(52.0, 96.0),
            Depth::Near => rng.range(88.0, 156.0),
        };

        // Weighted to the right, because that is where the page has room for
        // them. The words sit on the left and want the dark. Raising a number
        // between 0 and 1 to a power below 1 pushes it up, so this leans right
        // without ever clearing the left entirely.
        let left = 108.0 * rng.unit().powf(0.55) - 9.0;

        // Parallax: the nearer a rock is, the faster it crosses. This is the
        // whole of the depth illusion, and it only works if the order holds.
        let speed = match depth {
            Depth::Far => rng.range(1.6, 4.0),
            Depth::Middle => rng.range(4.5, 9.0),
            Depth::Near => rng.range(9.5, 17.0),
        };
        let heading = rng.range(0.0, TAU);

        out.push(Rock {
            left,
            top: rng.range(-6.0, 96.0),
            size,
            points: outline(&mut rng, 30.0, 46.0),
            facet: facet(&mut rng),
            gradient: gradients[(i % 3) as usize],
            depth,
            vx: heading.cos() * speed,
            vy: heading.sin() * speed,
            spin_rate: rng.range(-5.5, 5.5),
        });
    }
    out
}

pub fn stars() -> Vec<Star> {
    let mut rng = Rng(0x00C0_FFEE);
    (0..150)
        .map(|i| Star {
            left: rng.range(0.0, 100.0),
            top: rng.range(0.0, 100.0),
            size: rng.range(0.8, 2.3),
            alpha: rng.range(0.18, 0.92),
            twinkles: i % 3 == 0,
            secs: rng.range(2.6, 7.5),
            delay: rng.range(-7.0, 0.0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sky is the same sky every time, which is the only reason it is safe
    /// to draw it from random numbers at all.
    #[test]
    fn the_sky_is_settled() {
        let (a, b) = (rocks(), rocks());
        assert_eq!(a.len(), 22);
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.points, y.points);
            assert_eq!(x.left, y.left);
        }
        assert_eq!(stars().len(), 150);
    }

    /// Most of the rocks belong on the right, where the page has room. If this
    /// ever fails, they are hiding behind the words again.
    #[test]
    fn the_rocks_lean_right() {
        let rocks = rocks();
        let right = rocks.iter().filter(|r| r.left > 45.0).count();
        assert!(right * 2 > rocks.len(), "only {right} of {} lean right", rocks.len());
        assert!(rocks.iter().any(|r| r.left < 30.0), "the left is empty");
        assert!(rocks.iter().all(|r| r.left > -12.0 && r.left < 100.0));
    }

    /// Parallax is the whole of the depth illusion: if the far rocks ever
    /// outrun the near ones, the sky flattens.
    #[test]
    fn the_near_rocks_outrun_the_far_ones() {
        let rocks = rocks();
        let fastest = |d: Depth| {
            rocks
                .iter()
                .filter(|r| r.depth == d)
                .map(|r| (r.vx * r.vx + r.vy * r.vy).sqrt())
                .fold(0.0f32, f32::max)
        };
        let slowest = |d: Depth| {
            rocks
                .iter()
                .filter(|r| r.depth == d)
                .map(|r| (r.vx * r.vx + r.vy * r.vy).sqrt())
                .fold(f32::MAX, f32::min)
        };
        assert!(fastest(Depth::Far) < slowest(Depth::Middle));
        assert!(fastest(Depth::Middle) < slowest(Depth::Near));
        // And every one of them is actually going somewhere.
        assert!(rocks.iter().all(|r| r.vx.abs() + r.vy.abs() > 0.5));
    }

    /// A rock has to be a shape, not a line or a fold.
    #[test]
    fn a_rock_is_a_closed_shape() {
        for rock in rocks() {
            let corners = rock.points.split(' ').count();
            assert!((7..=11).contains(&corners), "{corners} corners");
            assert!(rock.size > 0.0);
        }
    }
}

/// One rock's motion, kept apart from the browser so it can be checked.
///
/// A stylesheet can only interpolate between two states it is handed. This is
/// not an interpolation: a rock holds a heading, keeps it, leaves the frame and
/// comes back around the opposite edge. That is a position being carried
/// forward, which is a thing a program does.
pub struct Drift {
    frac_left: f64,
    frac_top: f64,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    turned: f64,
    spin_rate: f64,
    /// Far enough out that a rock is gone from view before it is moved.
    margin: f64,
    placed: bool,
}

impl Drift {
    pub fn new(rock: &Rock) -> Self {
        Drift {
            frac_left: rock.left as f64 / 100.0,
            frac_top: rock.top as f64 / 100.0,
            x: 0.0,
            y: 0.0,
            vx: rock.vx as f64,
            vy: rock.vy as f64,
            turned: 0.0,
            spin_rate: rock.spin_rate as f64,
            margin: rock.size as f64 + 40.0,
            placed: false,
        }
    }

    /// Carries the rock forward by `secs`, wrapping it around a `width` by
    /// `height` sky. The first step only puts it where the stylesheet already
    /// says it is.
    pub fn step(&mut self, secs: f64, width: f64, height: f64) {
        if !self.placed {
            self.x = self.frac_left * width;
            self.y = self.frac_top * height;
            self.placed = true;
        }

        self.x += self.vx * secs;
        self.y += self.vy * secs;
        self.turned += self.spin_rate * secs;

        // Off one edge, on at the other.
        if self.x > width + self.margin {
            self.x = -self.margin;
        } else if self.x < -self.margin {
            self.x = width + self.margin;
        }
        if self.y > height + self.margin {
            self.y = -self.margin;
        } else if self.y < -self.margin {
            self.y = height + self.margin;
        }
    }

    /// Where it is now, said as an offset from where the stylesheet put it.
    pub fn transform(&self, width: f64, height: f64) -> String {
        format!(
            "translate3d({:.1}px, {:.1}px, 0) rotate({:.2}deg)",
            self.x - self.frac_left * width,
            self.y - self.frac_top * height,
            self.turned
        )
    }
}

/// Moves the rocks, one frame at a time. Everything it decides is `Drift`'s;
/// this only reads the clock, reads the size of the sky, and writes a
/// transform.
#[cfg(target_arch = "wasm32")]
pub fn animate() {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    let Some(win) = web_sys::window() else { return };

    // A reader who has asked for less movement is asking for none of this.
    if let Ok(Some(mq)) = win.match_media("(prefers-reduced-motion: reduce)") {
        if mq.matches() {
            return;
        }
    }

    let Some(doc) = win.document() else { return };
    let Ok(nodes) = doc.query_selector_all(".rock") else { return };
    let Ok(Some(hero)) = doc.query_selector(".hero") else { return };

    let specs = rocks();
    if nodes.length() as usize != specs.len() {
        return;
    }

    let mut bodies: Vec<(HtmlElement, Drift)> = Vec::with_capacity(specs.len());
    for (i, spec) in specs.iter().enumerate() {
        let Some(el) = nodes
            .get(i as u32)
            .and_then(|n| n.dyn_into::<HtmlElement>().ok())
        else {
            return;
        };
        bodies.push((el, Drift::new(spec)));
    }

    let bodies = Rc::new(RefCell::new(bodies));
    let frame: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next = frame.clone();
    let last = Rc::new(RefCell::new(f64::NAN));

    *next.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
        let Some(win) = web_sys::window() else { return };

        let secs = {
            let mut last = last.borrow_mut();
            let previous = *last;
            *last = now;
            // The first frame has nothing to measure from, and the frame after
            // a backgrounded tab has far too much. Neither is a real step.
            if previous.is_nan() {
                0.0
            } else {
                ((now - previous) / 1000.0).clamp(0.0, 0.05)
            }
        };

        let width = hero.client_width() as f64;
        let height = hero.client_height() as f64;

        // Nothing to do while the sky is scrolled off the top of the window.
        let showing = win.scroll_y().unwrap_or(0.0) <= height;

        if width > 0.0 && showing {
            for (el, drift) in bodies.borrow_mut().iter_mut() {
                drift.step(secs, width, height);
                let _ = el
                    .style()
                    .set_property("transform", &drift.transform(width, height));
            }
        }

        if let Some(cb) = frame.borrow().as_ref() {
            let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));

    // The closure holds the Rc that holds the closure, which is what keeps it
    // alive for as long as the page is.
    let first = next.borrow();
    if let Some(cb) = first.as_ref() {
        let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

/// Off the browser there is nothing to move.
#[cfg(not(target_arch = "wasm32"))]
pub fn animate() {}

#[cfg(test)]
mod drift_tests {
    use super::*;

    const W: f64 = 1440.0;
    const H: f64 = 820.0;

    fn one(vx: f32, vy: f32, left: f32, top: f32) -> Drift {
        let mut rock = rocks().pop().unwrap();
        rock.vx = vx;
        rock.vy = vy;
        rock.left = left;
        rock.top = top;
        rock.size = 60.0;
        Drift::new(&rock)
    }

    /// The first step only places a rock; it does not also move it.
    #[test]
    fn the_first_step_places_it_where_the_stylesheet_said() {
        let mut d = one(10.0, 0.0, 50.0, 25.0);
        d.step(0.0, W, H);
        assert_eq!(d.transform(W, H), "translate3d(0.0px, 0.0px, 0) rotate(0.00deg)");
    }

    /// A rock goes where its heading says, at the speed its heading says.
    #[test]
    fn it_carries_its_position_forward() {
        let mut d = one(20.0, -8.0, 50.0, 50.0);
        d.step(0.0, W, H);
        for _ in 0..10 {
            d.step(0.1, W, H);
        }
        // One second at 20 across and 8 up.
        assert!((d.x - (0.5 * W + 20.0)).abs() < 1e-6, "x was {}", d.x);
        assert!((d.y - (0.5 * H - 8.0)).abs() < 1e-6, "y was {}", d.y);
    }

    /// Off one edge, on at the other — and never in view while it happens.
    #[test]
    fn it_comes_back_around_the_far_side() {
        let mut d = one(400.0, 0.0, 95.0, 50.0);
        d.step(0.0, W, H);
        let mut wrapped = false;
        for _ in 0..200 {
            let before = d.x;
            d.step(0.05, W, H);
            if d.x < before {
                wrapped = true;
                // It reappears off the left edge, not inside the frame.
                assert!(d.x < 0.0, "reappeared at {}, which is on screen", d.x);
                break;
            }
        }
        assert!(wrapped, "it never came back around");
    }

    #[test]
    fn it_wraps_upwards_too() {
        let mut d = one(0.0, -400.0, 50.0, 5.0);
        d.step(0.0, W, H);
        for _ in 0..200 {
            d.step(0.05, W, H);
            if d.y > H {
                return;
            }
        }
        panic!("it left the top and never came back at the bottom");
    }

    /// Turning is time, not frames: the same second of motion has to land in
    /// the same place however it is chopped up.
    #[test]
    fn the_same_second_lands_in_the_same_place() {
        let (mut coarse, mut fine) = (one(30.0, 12.0, 40.0, 40.0), one(30.0, 12.0, 40.0, 40.0));
        coarse.step(0.0, W, H);
        fine.step(0.0, W, H);
        coarse.step(0.05, W, H);
        for _ in 0..5 {
            fine.step(0.01, W, H);
        }
        assert!((coarse.x - fine.x).abs() < 1e-9);
        assert!((coarse.turned - fine.turned).abs() < 1e-9);
    }
}
