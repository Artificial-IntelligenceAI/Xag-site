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
    settle(&mut out);
    out
}

/// The viewport the starting layout is worked out against. A rock's place is a
/// percentage but its size is in pixels, so whether two of them overlap is a
/// question that needs a window to be asked in. This is a middling one.
const NOMINAL: (f32, f32) = (1440.0, 820.0);

/// Pushes apart any two rocks that begin on top of each other.
///
/// Without this the page opens mid-explosion: two rocks laid down overlapping
/// are touching on the first frame and burst before anybody has seen them,
/// which reads as a glitch rather than as an event.
fn settle(rocks: &mut [Rock]) {
    let (w, h) = NOMINAL;

    for _ in 0..24 {
        let mut moved = false;

        for i in 0..rocks.len() {
            for j in (i + 1)..rocks.len() {
                if rocks[i].depth != rocks[j].depth {
                    continue;
                }

                let centre = |r: &Rock| {
                    (r.left / 100.0 * w + r.size / 2.0, r.top / 100.0 * h + r.size / 2.0)
                };
                let (ax, ay) = centre(&rocks[i]);
                let (bx, by) = centre(&rocks[j]);
                let apart = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                // A margin over touching, so drifting does not put them back
                // together within the first second either.
                let wanted = (rocks[i].size + rocks[j].size) * REACH_F32 + 24.0;

                if apart >= wanted {
                    continue;
                }

                // Straight up if they are exactly on top of one another, which
                // is otherwise a direction nobody can compute.
                let (dx, dy) = if apart < 0.001 { (0.0, 1.0) } else { ((ax - bx) / apart, (ay - by) / apart) };
                let shove = (wanted - apart) / 2.0;

                rocks[i].left += dx * shove / w * 100.0;
                rocks[i].top += dy * shove / h * 100.0;
                rocks[j].left -= dx * shove / w * 100.0;
                rocks[j].top -= dy * shove / h * 100.0;
                moved = true;
            }
        }

        if !moved {
            break;
        }
    }
}

/// `REACH` as the f32 this works in.
const REACH_F32: f32 = 0.46;

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

    /// The page must not open mid-explosion.
    #[test]
    fn nothing_starts_on_top_of_anything_else() {
        let rocks = rocks();
        let (w, h) = NOMINAL;
        for i in 0..rocks.len() {
            for j in (i + 1)..rocks.len() {
                if rocks[i].depth != rocks[j].depth {
                    continue;
                }
                let centre = |r: &Rock| {
                    (r.left / 100.0 * w + r.size / 2.0, r.top / 100.0 * h + r.size / 2.0)
                };
                let (ax, ay) = centre(&rocks[i]);
                let (bx, by) = centre(&rocks[j]);
                let apart = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                let touching = (rocks[i].size + rocks[j].size) * REACH_F32;
                assert!(
                    apart > touching,
                    "rocks {i} and {j} start {apart:.0} apart and touch at {touching:.0}"
                );
            }
        }
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

/// How many pieces a rock comes apart into.
const SHARDS_PER_ROCK: usize = 7;

/// Enough for three collisions happening at once. Past that the oldest pieces
/// are still on screen and nobody is counting.
pub const SHARD_POOL: usize = SHARDS_PER_ROCK * 6;

pub const FLASH_POOL: usize = 4;

/// How long a rock is gone for before it comes back in from an edge.
const GONE_FOR: f64 = 1.2;

/// A rock's outline reaches 46 units in a box of 100, so this is the radius of
/// the circle that holds it.
const REACH: f64 = 0.46;

/// One rock's motion, kept apart from the browser so it can be checked.
///
/// A stylesheet can only interpolate between two states it is handed. This is
/// not an interpolation: a rock holds a heading, keeps it, leaves the frame and
/// comes back around the opposite edge. That is a position being carried
/// forward, which is a thing a program does.
pub struct Drift {
    frac_left: f64,
    frac_top: f64,
    pub x: f64,
    pub y: f64,
    vx: f64,
    vy: f64,
    pub turned: f64,
    spin_rate: f64,
    size: f64,
    depth: Depth,
    /// Far enough out that a rock is gone from view before it is moved.
    margin: f64,
    placed: bool,
    /// Seconds left before it comes back. Zero means it is here.
    gone_for: f64,
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
            size: rock.size as f64,
            depth: rock.depth,
            margin: rock.size as f64 + 40.0,
            placed: false,
            gone_for: 0.0,
        }
    }

    /// Here to be seen and to be hit. A rock in pieces is neither.
    pub fn present(&self) -> bool {
        self.gone_for <= 0.0
    }

    pub fn centre(&self) -> (f64, f64) {
        (self.x + self.size / 2.0, self.y + self.size / 2.0)
    }

    pub fn reach(&self) -> f64 {
        self.size * REACH
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

        if self.gone_for > 0.0 {
            self.gone_for -= secs;
            return;
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

    /// Takes it out of the sky, and arranges for it to come back.
    fn shatter(&mut self) {
        self.gone_for = GONE_FOR;
    }

    /// Back in from an edge on a new heading, keeping the speed its distance
    /// gave it so the parallax still holds.
    fn returns(&mut self, rng: &mut Rng, width: f64, height: f64) {
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        let heading = rng.range(0.0, TAU) as f64;
        self.vx = heading.cos() * speed;
        self.vy = heading.sin() * speed;
        self.turned = 0.0;

        // On whichever edge it is now heading away from, so it drifts inwards
        // rather than straight back out again.
        if self.vx.abs() > self.vy.abs() {
            self.x = if self.vx > 0.0 { -self.margin } else { width + self.margin };
            self.y = rng.range(0.0, height as f32) as f64;
        } else {
            self.y = if self.vy > 0.0 { -self.margin } else { height + self.margin };
            self.x = rng.range(0.0, width as f32) as f64;
        }
    }
}

/// A piece of a rock that has come apart.
pub struct Shard {
    pub x: f64,
    pub y: f64,
    vx: f64,
    vy: f64,
    pub turned: f64,
    spin_rate: f64,
    pub size: f64,
    life: f64,
    full_life: f64,
}

impl Shard {
    fn idle() -> Self {
        Shard {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            turned: 0.0,
            spin_rate: 0.0,
            size: 0.0,
            life: 0.0,
            full_life: 1.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.life > 0.0
    }

    /// Bright when it is new and gone by the end, which is the whole of the
    /// explosion as far as the eye is concerned.
    pub fn fade(&self) -> f64 {
        (self.life / self.full_life).clamp(0.0, 1.0)
    }

    fn step(&mut self, secs: f64) {
        if !self.alive() {
            return;
        }
        self.life -= secs;
        self.x += self.vx * secs;
        self.y += self.vy * secs;
        self.turned += self.spin_rate * secs;
        // Thrown outwards and then slowing, so the burst reads as a burst
        // rather than as a steady drift away.
        let drag = (1.0 - 1.9 * secs).max(0.0);
        self.vx *= drag;
        self.vy *= drag;
    }
}

/// The light of an impact, which is over before the pieces are.
pub struct Flash {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    life: f64,
    full_life: f64,
}

impl Flash {
    fn idle() -> Self {
        Flash { x: 0.0, y: 0.0, size: 0.0, life: 0.0, full_life: 1.0 }
    }

    pub fn alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn fade(&self) -> f64 {
        (self.life / self.full_life).clamp(0.0, 1.0)
    }

    /// Opens outwards as it goes.
    pub fn spread(&self) -> f64 {
        0.4 + 1.6 * (1.0 - self.fade())
    }

    fn step(&mut self, secs: f64) {
        if self.alive() {
            self.life -= secs;
        }
    }
}

/// Everything in the sky, and what happens when two bits of it meet.
pub struct Sky {
    pub drifts: Vec<Drift>,
    pub shards: Vec<Shard>,
    pub flashes: Vec<Flash>,
    rng: Rng,
}

impl Default for Sky {
    fn default() -> Self {
        Self::new()
    }
}

impl Sky {
    pub fn new() -> Self {
        Sky {
            drifts: rocks().iter().map(Drift::new).collect(),
            shards: (0..SHARD_POOL).map(|_| Shard::idle()).collect(),
            flashes: (0..FLASH_POOL).map(|_| Flash::idle()).collect(),
            rng: Rng(0x0BAD_5EED),
        }
    }

    /// Carries the whole sky forward one frame.
    pub fn step(&mut self, secs: f64, width: f64, height: f64) {
        for drift in self.drifts.iter_mut() {
            drift.step(secs, width, height);
        }

        for hit in self.collisions() {
            self.burst(hit, width, height);
        }

        for shard in self.shards.iter_mut() {
            shard.step(secs);
        }
        for flash in self.flashes.iter_mut() {
            flash.step(secs);
        }
    }

    /// Which pairs are touching.
    ///
    /// Only rocks at the same distance can meet: a speck far away and a boulder
    /// close up are nowhere near each other, whatever the screen says, and
    /// bursting them together would say the sky is flat.
    fn collisions(&self) -> Vec<(usize, usize)> {
        let mut hits = Vec::new();
        for i in 0..self.drifts.len() {
            for j in (i + 1)..self.drifts.len() {
                let (a, b) = (&self.drifts[i], &self.drifts[j]);
                if !a.present() || !b.present() || a.depth != b.depth {
                    continue;
                }
                let (ax, ay) = a.centre();
                let (bx, by) = b.centre();
                let touching = a.reach() + b.reach();
                let (dx, dy) = (ax - bx, ay - by);
                if dx * dx + dy * dy <= touching * touching {
                    hits.push((i, j));
                }
            }
        }
        hits
    }

    fn burst(&mut self, (i, j): (usize, usize), width: f64, height: f64) {
        // A rock caught by two collisions in one frame only comes apart once.
        if !self.drifts[i].present() || !self.drifts[j].present() {
            return;
        }

        let (ax, ay) = self.drifts[i].centre();
        let (bx, by) = self.drifts[j].centre();
        let (mx, my) = ((ax + bx) / 2.0, (ay + by) / 2.0);
        let biggest = self.drifts[i].size.max(self.drifts[j].size);

        self.light(mx, my, biggest);

        for k in [i, j] {
            let (cx, cy) = self.drifts[k].centre();
            let size = self.drifts[k].size;
            self.scatter(cx, cy, size);
            self.drifts[k].shatter();

            // Put where it will be when it comes back, which it does not do
            // until `gone_for` runs out — so moving it now is unseen.
            let seed = self.rng.bits();
            let mut rng = Rng(seed);
            self.drifts[k].returns(&mut rng, width, height);
        }
    }

    fn light(&mut self, x: f64, y: f64, size: f64) {
        if let Some(flash) = self.flashes.iter_mut().find(|f| !f.alive()) {
            flash.x = x;
            flash.y = y;
            flash.size = size * 1.5;
            flash.full_life = 0.42;
            flash.life = 0.42;
        }
    }

    fn scatter(&mut self, x: f64, y: f64, size: f64) {
        for _ in 0..SHARDS_PER_ROCK {
            let heading = self.rng.range(0.0, TAU) as f64;
            let speed = self.rng.range(70.0, 240.0) as f64;
            let life = self.rng.range(0.7, 1.35) as f64;
            let piece = self.rng.range(0.16, 0.34) as f64 * size;
            let spin = self.rng.range(-260.0, 260.0) as f64;

            let Some(shard) = self.shards.iter_mut().find(|s| !s.alive()) else {
                return; // every piece is already in the air
            };
            shard.x = x;
            shard.y = y;
            shard.vx = heading.cos() * speed;
            shard.vy = heading.sin() * speed;
            shard.turned = 0.0;
            shard.spin_rate = spin;
            shard.size = piece;
            shard.full_life = life;
            shard.life = life;
        }
    }
}

/// A shape for each piece in the pool, worked out once so the markup can hold
/// them and nothing has to build an outline while things are moving.
pub fn shard_shapes() -> Vec<String> {
    let mut rng = Rng(0x5EED_1234);
    (0..SHARD_POOL).map(|_| outline(&mut rng, 26.0, 48.0)).collect()
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// Kept between stops, so the sky is where it was rather than where it
    /// started when it comes back.
    static SKY: std::cell::RefCell<Option<Sky>> = const { std::cell::RefCell::new(None) };
    /// Whether a frame loop is already going. Two of them would step the same
    /// sky twice a frame and everything would move at double speed.
    static LOOPING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Set by the theme. The running loop notices, puts everything away, and
    /// stops asking for frames.
    static PAUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Stops the sky. The loop ends rather than idling: nothing is stepped, and no
/// further frames are asked for.
#[cfg(target_arch = "wasm32")]
pub fn pause() {
    PAUSED.with(|p| p.set(true));
}

#[cfg(target_arch = "wasm32")]
pub fn resume() {
    PAUSED.with(|p| p.set(false));
    animate();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pause() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn resume() {}

/// Moves the sky, one frame at a time. Everything it decides is `Sky`'s; this
/// only reads the clock, reads the size of the window, and writes styles.
#[cfg(target_arch = "wasm32")]
pub fn animate() {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    // Already going, or asked to stop. Either way there is nothing to start.
    if LOOPING.with(|l| l.get()) || PAUSED.with(|p| p.get()) {
        return;
    }

    let Some(win) = web_sys::window() else { return };

    // A reader who has asked for less movement is asking for none of this.
    if let Ok(Some(mq)) = win.match_media("(prefers-reduced-motion: reduce)") {
        if mq.matches() {
            return;
        }
    }

    let Some(doc) = win.document() else { return };
    let Ok(Some(hero)) = doc.query_selector(".hero") else { return };

    fn collect(doc: &web_sys::Document, selector: &str, wanted: usize) -> Option<Vec<HtmlElement>> {
        let nodes = doc.query_selector_all(selector).ok()?;
        if nodes.length() as usize != wanted {
            return None;
        }
        (0..wanted)
            .map(|i| nodes.get(i as u32).and_then(|n| n.dyn_into::<HtmlElement>().ok()))
            .collect()
    }

    let rocks_wanted = SKY.with(|s| {
        let mut s = s.borrow_mut();
        s.get_or_insert_with(Sky::new).drifts.len()
    });
    let Some(rock_els) = collect(&doc, ".rock", rocks_wanted) else { return };
    let Some(shard_els) = collect(&doc, ".shard", SHARD_POOL) else { return };
    let Some(flash_els) = collect(&doc, ".flash", FLASH_POOL) else { return };

    LOOPING.with(|l| l.set(true));
    let frame: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next = frame.clone();
    let last = Rc::new(RefCell::new(f64::NAN));

    *next.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
        let Some(win) = web_sys::window() else { return };

        // Asked to stop: put everything away and do not ask for another frame.
        if PAUSED.with(|p| p.get()) {
            for el in rock_els.iter().chain(shard_els.iter()).chain(flash_els.iter()) {
                let _ = el.style().set_property("display", "none");
            }
            LOOPING.with(|l| l.set(false));
            return;
        }

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
            SKY.with(|cell| {
            let mut sky = cell.borrow_mut();
            let Some(sky) = sky.as_mut() else { return };
            sky.step(secs, width, height);

            for (el, drift) in rock_els.iter().zip(sky.drifts.iter()) {
                let style = el.style();
                if drift.present() {
                    let _ = style.set_property("display", "block");
                    let _ = style.set_property("transform", &drift.transform(width, height));
                } else {
                    let _ = style.set_property("display", "none");
                }
            }

            for (el, shard) in shard_els.iter().zip(sky.shards.iter()) {
                let style = el.style();
                if shard.alive() {
                    let _ = style.set_property("display", "block");
                    let _ = style.set_property(
                        "transform",
                        &format!(
                            "translate3d({:.1}px, {:.1}px, 0) rotate({:.1}deg)",
                            shard.x - shard.size / 2.0,
                            shard.y - shard.size / 2.0,
                            shard.turned
                        ),
                    );
                    let _ = style.set_property("width", &format!("{:.1}px", shard.size));
                    let _ = style.set_property("height", &format!("{:.1}px", shard.size));
                    let _ = style.set_property("opacity", &format!("{:.3}", shard.fade()));
                } else {
                    let _ = style.set_property("display", "none");
                }
            }

            for (el, flash) in flash_els.iter().zip(sky.flashes.iter()) {
                let style = el.style();
                if flash.alive() {
                    let _ = style.set_property("display", "block");
                    let _ = style.set_property(
                        "transform",
                        &format!(
                            "translate3d({:.1}px, {:.1}px, 0) scale({:.2})",
                            flash.x - flash.size / 2.0,
                            flash.y - flash.size / 2.0,
                            flash.spread()
                        ),
                    );
                    let _ = style.set_property("width", &format!("{:.1}px", flash.size));
                    let _ = style.set_property("height", &format!("{:.1}px", flash.size));
                    let _ = style.set_property("opacity", &format!("{:.3}", flash.fade()));
                } else {
                    let _ = style.set_property("display", "none");
                }
            }
            });
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
mod sky_tests {
    use super::*;

    const W: f64 = 1440.0;
    const H: f64 = 820.0;

    fn put(drift: &mut Drift, x: f64, y: f64, depth: Depth, size: f64) {
        drift.placed = true;
        drift.x = x;
        drift.y = y;
        drift.depth = depth;
        drift.size = size;
        drift.margin = size + 40.0;
        drift.vx = 0.0;
        drift.vy = 0.0;
        drift.gone_for = 0.0;
    }

    /// Everything spread out on a grid and too small to touch, so a test can
    /// then place the two it cares about.
    fn quiet_sky() -> Sky {
        let mut sky = Sky::new();
        for (i, drift) in sky.drifts.iter_mut().enumerate() {
            let x = 50.0 + (i % 6) as f64 * 200.0;
            let y = 50.0 + (i / 6) as f64 * 200.0;
            put(drift, x, y, Depth::Far, 4.0);
        }
        sky
    }

    fn alive_shards(sky: &Sky) -> usize {
        sky.shards.iter().filter(|s| s.alive()).count()
    }

    #[test]
    fn nothing_happens_in_a_quiet_sky() {
        let mut sky = quiet_sky();
        sky.step(0.016, W, H);
        assert_eq!(alive_shards(&sky), 0);
        assert!(sky.drifts.iter().all(|d| d.present()));
    }

    #[test]
    fn two_rocks_that_meet_come_apart() {
        let mut sky = quiet_sky();
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 420.0, 400.0, Depth::Near, 100.0);

        sky.step(0.016, W, H);

        assert!(!sky.drifts[0].present(), "the first one is still here");
        assert!(!sky.drifts[1].present(), "the second one is still here");
        assert_eq!(alive_shards(&sky), SHARDS_PER_ROCK * 2);
        assert_eq!(sky.flashes.iter().filter(|f| f.alive()).count(), 1);
    }

    /// A speck far away and a boulder close up are nowhere near each other,
    /// whatever the screen says.
    #[test]
    fn rocks_at_different_distances_pass_through_each_other() {
        let mut sky = quiet_sky();
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 420.0, 400.0, Depth::Middle, 100.0);

        sky.step(0.016, W, H);

        assert!(sky.drifts[0].present());
        assert!(sky.drifts[1].present());
        assert_eq!(alive_shards(&sky), 0);
    }

    /// Touching is edge to edge, not centre to centre.
    #[test]
    fn rocks_that_only_nearly_meet_do_not() {
        let mut sky = quiet_sky();
        // Reaches are 46 each, so 93 apart is clear by a whisker.
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 493.0, 400.0, Depth::Near, 100.0);

        sky.step(0.016, W, H);

        assert!(sky.drifts[0].present(), "they were not touching");
        assert_eq!(alive_shards(&sky), 0);
    }

    #[test]
    fn what_came_apart_comes_back_from_an_edge() {
        let mut sky = quiet_sky();
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 420.0, 400.0, Depth::Near, 100.0);
        sky.step(0.016, W, H);
        assert!(!sky.drifts[0].present());

        for _ in 0..((GONE_FOR / 0.016) as usize + 4) {
            sky.step(0.016, W, H);
        }

        assert!(sky.drifts[0].present(), "it never came back");
        let (cx, cy) = sky.drifts[0].centre();
        let out = cx < 0.0 || cx > W || cy < 0.0 || cy > H;
        assert!(out, "it came back at {cx},{cy}, which is in plain view");
    }

    #[test]
    fn the_pieces_do_not_last() {
        let mut sky = quiet_sky();
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 420.0, 400.0, Depth::Near, 100.0);
        sky.step(0.016, W, H);
        assert!(alive_shards(&sky) > 0);

        for _ in 0..120 {
            sky.step(0.016, W, H);
        }
        assert_eq!(alive_shards(&sky), 0, "something is still in the air");
        assert!(sky.flashes.iter().all(|f| !f.alive()));
    }

    /// A piece thrown outwards leaves, and fades as it goes.
    #[test]
    fn a_piece_leaves_and_fades() {
        let mut sky = quiet_sky();
        put(&mut sky.drifts[0], 400.0, 400.0, Depth::Near, 100.0);
        put(&mut sky.drifts[1], 420.0, 400.0, Depth::Near, 100.0);
        sky.step(0.016, W, H);

        let (sx, sy) = {
            let s = sky.shards.iter().find(|s| s.alive()).unwrap();
            (s.x, s.y)
        };
        let before = sky.shards.iter().find(|s| s.alive()).unwrap().fade();

        for _ in 0..20 {
            sky.step(0.016, W, H);
        }

        let s = sky.shards.iter().find(|s| s.alive()).expect("all gone too soon");
        assert!((s.x - sx).abs() + (s.y - sy).abs() > 1.0, "it never moved");
        assert!(s.fade() < before, "it never faded");
    }

    /// More collisions than there are pieces must not panic or overrun.
    #[test]
    fn the_pool_holds_when_everything_meets_at_once() {
        let mut sky = quiet_sky();
        for i in 0..sky.drifts.len() {
            put(&mut sky.drifts[i], 400.0, 400.0, Depth::Near, 100.0);
        }
        sky.step(0.016, W, H);
        assert!(alive_shards(&sky) <= SHARD_POOL);
    }
}

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
        assert!((d.x - (0.5 * W + 20.0)).abs() < 1e-6, "x was {}", d.x);
        assert!((d.y - (0.5 * H - 8.0)).abs() < 1e-6, "y was {}", d.y);
    }

    /// Off one edge, on at the other — and never in view while it happens.
    #[test]
    fn it_comes_back_around_the_far_side() {
        let mut d = one(400.0, 0.0, 95.0, 50.0);
        d.step(0.0, W, H);
        for _ in 0..200 {
            let before = d.x;
            d.step(0.05, W, H);
            if d.x < before {
                assert!(d.x < 0.0, "reappeared at {}, which is on screen", d.x);
                return;
            }
        }
        panic!("it never came back around");
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

#[cfg(test)]
mod frequency {
    use super::*;

    /// A collision nobody ever sees is a feature that is not there, and one
    /// happening constantly is a mess. This runs the sky forward and counts, so
    /// the rate is a measurement rather than a hope.
    #[test]
    fn how_often_two_rocks_meet() {
        let (w, h) = (1440.0, 820.0);
        let mut sky = Sky::new();
        // Settle it first, so the count is of a sky in motion rather than of
        // wherever the rocks happened to be drawn.
        for _ in 0..600 {
            sky.step(1.0 / 60.0, w, h);
        }

        let minutes = 5.0;
        let frames = (minutes * 60.0 * 60.0) as usize;
        let mut bursts = 0usize;
        let mut gone = vec![false; sky.drifts.len()];

        for _ in 0..frames {
            sky.step(1.0 / 60.0, w, h);
            for (i, drift) in sky.drifts.iter().enumerate() {
                let now_gone = !drift.present();
                if now_gone && !gone[i] {
                    bursts += 1;
                }
                gone[i] = now_gone;
            }
        }

        // Measured at 52 in five minutes, which is a collision about every
        // eleven seconds. The band is wide because the exact figure is not the
        // point — being seen at all, and not constantly, is.
        assert!(
            (12..=200).contains(&bursts),
            "{bursts} rocks broke up in {minutes} minutes, which is either \
             never or nonstop"
        );
    }
}
