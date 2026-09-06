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
    pub secs: f32,
    pub delay: f32,
    pub dx: f32,
    pub dy: f32,
    pub spin: f32,
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

        // The nearer a rock is, the further it appears to move.
        let travel = match depth {
            Depth::Far => 6.0,
            Depth::Middle => 13.0,
            Depth::Near => 22.0,
        };

        out.push(Rock {
            left,
            top: rng.range(-6.0, 96.0),
            size,
            points: outline(&mut rng, 30.0, 46.0),
            facet: facet(&mut rng),
            gradient: gradients[(i % 3) as usize],
            depth,
            secs: rng.range(15.0, 38.0),
            delay: rng.range(-20.0, 0.0),
            dx: rng.range(-travel, travel),
            dy: rng.range(-travel, travel),
            spin: rng.range(-16.0, 16.0),
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
