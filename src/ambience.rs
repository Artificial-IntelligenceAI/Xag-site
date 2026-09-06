//! The sound of the space the rocks are drifting in.
//!
//! Nothing here is a recording and nothing here is a song. There is no sample
//! to license, no file to serve, and no melody to get stuck in anyone's head:
//! every sound is worked out from scratch when it is asked for.
//!
//! What makes something sound like a place rather than a tune is that it never
//! resolves and never repeats. The layers below are moved by oscillators whose
//! rates share no common multiple worth waiting for, so the combination does
//! not come back around inside a sitting.
//!
//! It is off until it is asked for. A page that makes noise at someone who did
//! not ask is a page they close.

/// The drone, and how loud each part of it is: a root, a fifth and an octave,
/// which is the spacing that beats slowly rather than arguing.
///
/// The root is 110 and not 55. A laptop speaker reproduces almost nothing below
/// about 180Hz, so a drone written an octave down is a drone most people cannot
/// hear at all — the first version of this put nearly all its energy under
/// 80Hz and was silent on anything without a woofer. The 55 is kept underneath
/// at a low level for the machines that can render it, where it adds weight
/// rather than pitch.
const DRONE: [(f32, f32); 4] = [
    (55.0, 0.18),
    (110.0, 0.26),
    (164.81, 0.22),
    (220.0, 0.14),
];

/// Partials that fade in and out over the drone, in the range a small speaker
/// is actually good at.
const PARTIALS: [f32; 4] = [329.63, 440.0, 659.26, 880.0];

/// Rates for the slow movement, in hertz. They are deliberately awkward
/// against each other: 0.013 and 0.017 and 0.023 and 0.031 are all primes over
/// a thousand, so the pattern they make together takes days to come round.
const LFO_RATES: [f32; 4] = [0.013, 0.017, 0.023, 0.031];

/// A minor pentatonic, four octaves up. A ping can land on any of these
/// without ever implying the next one, which is what keeps it from becoming a
/// melody.
const PINGS: [f32; 5] = [880.0, 1046.5, 1174.7, 1318.5, 1568.0];

/// Where the mix is meant to land, measured at the destination: about -23dBFS
/// RMS, which is present without being an event. It is written down because it
/// was got wrong once by a factor of thirty.
pub const TARGET_DBFS: f32 = -23.0;

/// The same xorshift the sky is drawn with.
pub struct Rng(pub u32);

impl Rng {
    pub fn bits(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    pub fn unit(&mut self) -> f32 {
        (self.bits() >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.unit() * (hi - lo)
    }
}

/// Brown noise: white noise integrated, so there is far more of it low down
/// than high up. White noise hisses like a broken speaker; this rumbles, which
/// is what a hull does.
///
/// The running sum is pulled back towards zero a little at every step, because
/// integrating noise otherwise wanders off and takes the whole signal with it.
pub fn brown_noise(samples: usize, seed: u32) -> Vec<f32> {
    let mut rng = Rng(seed);
    let mut out = Vec::with_capacity(samples);
    let mut running = 0.0f32;

    for _ in 0..samples {
        let white = rng.range(-1.0, 1.0);
        running = (running + white * 0.02) * 0.995;
        out.push(running);
    }

    // Brought up to a known level, so the gain staging downstream means
    // something regardless of how the walk happened to go.
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        for s in out.iter_mut() {
            *s /= peak;
        }
    }
    out
}

/// One channel of a reverb tail: noise under a curve that falls away.
///
/// Convolving anything with this scatters it into a room, and how long the
/// curve takes to fall is how big the room sounds. Six seconds is not a room —
/// it is the inside of something with no walls in reach.
pub fn impulse_response(samples: usize, decay: f32, seed: u32) -> Vec<f32> {
    let mut rng = Rng(seed);
    let n = samples.max(1) as f32;

    (0..samples)
        .map(|i| {
            let through = i as f32 / n;
            // A power curve rather than a straight line, because loudness is
            // not heard in equal steps.
            let envelope = (1.0 - through).powf(decay);
            rng.range(-1.0, 1.0) * envelope
        })
        .collect()
}

// ─────────────────────────── the browser half ───────────────────────────

#[cfg(target_arch = "wasm32")]
mod live {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::{
        AudioContext, BiquadFilterType, GainNode, OscillatorType,
    };

    /// How loud it gets once it has faded all the way in. Ambience is meant to
    /// be noticed on leaving rather than on arriving.
    const LEVEL: f32 = 0.34;
    const FADE: f64 = 3.0;

    thread_local! {
        static RUNNING: RefCell<Option<Voice>> = const { RefCell::new(None) };
    }

    pub struct Voice {
        ctx: AudioContext,
        master: GainNode,
        stop: Rc<RefCell<bool>>,
    }

    fn buffer_from(
        ctx: &AudioContext,
        channels: Vec<Vec<f32>>,
        rate: f32,
    ) -> Option<web_sys::AudioBuffer> {
        let frames = channels.first()?.len() as u32;
        let buf = ctx
            .create_buffer(channels.len() as u32, frames, rate)
            .ok()?;
        for (i, mut data) in channels.into_iter().enumerate() {
            buf.copy_to_channel(&mut data, i as i32).ok()?;
        }
        Some(buf)
    }

    /// An oscillator whose only job is to move one number slowly.
    fn slow_mover(ctx: &AudioContext, rate: f32, depth: f32) -> Option<GainNode> {
        let osc = ctx.create_oscillator().ok()?;
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(rate);
        let amount = ctx.create_gain().ok()?;
        amount.gain().set_value(depth);
        osc.connect_with_audio_node(&amount).ok()?;
        osc.start().ok()?;
        Some(amount)
    }

    pub fn start() -> Option<()> {
        let win = web_sys::window()?;
        let ctx = AudioContext::new().ok()?;
        // Safari in particular hands back a suspended context even inside a
        // gesture, and a suspended context is silence.
        let _ = ctx.resume();
        let rate = ctx.sample_rate();
        let now = ctx.current_time();

        let master = ctx.create_gain().ok()?;
        master.gain().set_value(0.0);
        master.connect_with_audio_node(&ctx.destination()).ok()?;
        // Faded in rather than switched on, so it arrives instead of starting.
        master.gain().set_value_at_time(0.0001, now).ok()?;
        master
            .gain()
            .exponential_ramp_to_value_at_time(LEVEL, now + FADE)
            .ok()?;

        // Everything goes through one very large room.
        let verb = ctx.create_convolver().ok()?;
        let tail = (rate * 6.0) as usize;
        let ir = buffer_from(
            &ctx,
            vec![
                impulse_response(tail, 2.6, 0x1234_5678),
                impulse_response(tail, 2.6, 0x8765_4321),
            ],
            rate,
        )?;
        verb.set_buffer(Some(&ir));
        verb.connect_with_audio_node(&master).ok()?;

        // A little of everything stays dry, or it turns to soup.
        let dry = ctx.create_gain().ok()?;
        dry.gain().set_value(0.55);
        dry.connect_with_audio_node(&master).ok()?;

        let bus = ctx.create_gain().ok()?;
        bus.connect_with_audio_node(&verb).ok()?;
        bus.connect_with_audio_node(&dry).ok()?;

        let mut rng = Rng(0xA5A5_1234);

        // ── the drone ──
        for (i, (hz, gain)) in DRONE.iter().enumerate() {
            let osc = ctx.create_oscillator().ok()?;
            // The top of the drone is a triangle, whose harmonics land at 660
            // and 1100 where a small speaker can carry them.
            osc.set_type(if i == 3 {
                OscillatorType::Triangle
            } else {
                OscillatorType::Sine
            });
            osc.frequency().set_value(*hz);

            // A few cents of wander, so two notes never sit exactly still
            // against each other.
            if let Some(drift) = slow_mover(&ctx, rng.range(0.03, 0.09), rng.range(1.5, 4.0)) {
                drift.connect_with_audio_param(&osc.detune()).ok()?;
            }

            let level = ctx.create_gain().ok()?;
            level.gain().set_value(*gain);
            osc.connect_with_audio_node(&level).ok()?;
            level.connect_with_audio_node(&bus).ok()?;
            osc.start().ok()?;
        }

        // ── partials that come and go ──
        for (i, hz) in PARTIALS.iter().enumerate() {
            let osc = ctx.create_oscillator().ok()?;
            osc.set_type(OscillatorType::Sine);
            osc.frequency().set_value(*hz);

            let level = ctx.create_gain().ok()?;
            // Sitting just above silence, so the mover below takes it under
            // and brings it back rather than pumping it.
            level.gain().set_value(0.085);
            if let Some(mover) = slow_mover(&ctx, LFO_RATES[i], 0.08) {
                mover.connect_with_audio_param(&level.gain()).ok()?;
            }
            osc.connect_with_audio_node(&level).ok()?;
            level.connect_with_audio_node(&bus).ok()?;
            osc.start().ok()?;
        }

        // ── the bed ──
        let noise = buffer_from(&ctx, vec![brown_noise((rate * 8.0) as usize, 0xC0FF_EE01)], rate)?;
        let source = ctx.create_buffer_source().ok()?;
        source.set_buffer(Some(&noise));
        source.set_loop(true);

        let hull = ctx.create_biquad_filter().ok()?;
        hull.set_type(BiquadFilterType::Lowpass);
        hull.frequency().set_value(900.0);
        hull.q().set_value(0.6);
        if let Some(mover) = slow_mover(&ctx, 0.019, 350.0) {
            mover.connect_with_audio_param(&hull.frequency()).ok()?;
        }

        let bed = ctx.create_gain().ok()?;
        bed.gain().set_value(0.34);
        source.connect_with_audio_node(&hull).ok()?;
        hull.connect_with_audio_node(&bed).ok()?;
        bed.connect_with_audio_node(&bus).ok()?;
        source.start().ok()?;

        // ── things happening a long way off ──
        let stop = Rc::new(RefCell::new(false));
        schedule_ping(&win, ctx.clone(), verb.clone().into(), stop.clone(), Rng(rng.bits()));

        RUNNING.with(|r| *r.borrow_mut() = Some(Voice { ctx, master, stop }));
        Some(())
    }

    /// One distant event, and then an arrangement to do it again later.
    fn schedule_ping(
        win: &web_sys::Window,
        ctx: AudioContext,
        out: web_sys::AudioNode,
        stop: Rc<RefCell<bool>>,
        mut rng: Rng,
    ) {
        let wait = rng.range(9_000.0, 26_000.0) as i32;
        let window = win.clone();

        let cb = Closure::once_into_js(move || {
            if *stop.borrow() {
                return;
            }

            let mut ring = || -> Option<()> {
                let now = ctx.current_time();
                let osc = ctx.create_oscillator().ok()?;
                osc.set_type(OscillatorType::Sine);
                let note = PINGS[(rng.bits() % PINGS.len() as u32) as usize];
                osc.frequency().set_value(note);

                let level = ctx.create_gain().ok()?;
                let life = rng.range(7.0, 12.0) as f64;
                level.gain().set_value_at_time(0.0001, now).ok()?;
                level
                    .gain()
                    .exponential_ramp_to_value_at_time(rng.range(0.09, 0.18), now + 0.4)
                    .ok()?;
                level
                    .gain()
                    .exponential_ramp_to_value_at_time(0.0001, now + life)
                    .ok()?;

                let pan = ctx.create_stereo_panner().ok()?;
                pan.pan().set_value(rng.range(-0.85, 0.85));

                osc.connect_with_audio_node(&level).ok()?;
                level.connect_with_audio_node(&pan).ok()?;
                pan.connect_with_audio_node(&out).ok()?;
                osc.start().ok()?;
                osc.stop_with_when(now + life + 0.1).ok()?;
                Some(())
            };
            let _ = ring();

            schedule_ping(&window, ctx, out, stop, rng);
        });

        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            wait,
        );
    }

    /// Faded out rather than cut, and then actually torn down.
    pub fn stop() {
        RUNNING.with(|r| {
            if let Some(voice) = r.borrow_mut().take() {
                *voice.stop.borrow_mut() = true;
                let now = voice.ctx.current_time();
                let _ = voice.master.gain().cancel_scheduled_values(now);
                let _ = voice
                    .master
                    .gain()
                    .set_value_at_time(voice.master.gain().value().max(0.0001), now);
                let _ = voice
                    .master
                    .gain()
                    .exponential_ramp_to_value_at_time(0.0001, now + 1.4);

                // Closed once the fade has actually finished.
                if let Some(win) = web_sys::window() {
                    let ctx = voice.ctx.clone();
                    let done = Closure::once_into_js(move || {
                        let _ = ctx.close();
                    });
                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                        done.as_ref().unchecked_ref(),
                        1600,
                    );
                }
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub fn start() {
    let _ = live::start();
}

#[cfg(target_arch = "wasm32")]
pub fn stop() {
    live::stop();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn stop() {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing may leave the range a sample is allowed to be in, or it is
    /// clipped on the way out and the room turns to crackle.
    #[test]
    fn every_sample_is_in_range() {
        for s in brown_noise(48_000, 7) {
            assert!(s.abs() <= 1.0, "brown noise reached {s}");
        }
        for s in impulse_response(48_000, 2.6, 9) {
            assert!(s.abs() <= 1.0, "impulse reached {s}");
        }
    }

    /// Brown noise is white noise integrated, so it has to hold far more low
    /// than high. Counting how often it crosses zero is the cheapest way to
    /// say so: white noise crosses about half the time, and this must not.
    #[test]
    fn the_bed_rumbles_rather_than_hisses() {
        let brown = brown_noise(48_000, 11);
        let crossings = brown.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
        let rate = crossings as f32 / brown.len() as f32;
        assert!(rate < 0.2, "crossed zero {rate} of the time, which is a hiss");
    }

    /// A reverb tail has to fall away. If the end is as loud as the beginning
    /// it is not a room, it is a wash.
    #[test]
    fn the_room_decays() {
        let ir = impulse_response(48_000, 2.6, 13);
        let energy = |half: &[f32]| half.iter().map(|s| s * s).sum::<f32>() / half.len() as f32;
        let (first, second) = ir.split_at(ir.len() / 2);
        assert!(
            energy(first) > energy(second) * 8.0,
            "first half {} against second {}",
            energy(first),
            energy(second)
        );
        assert!(ir.last().unwrap().abs() < 0.05, "it never actually stops");
    }

    /// The same seed is the same sound, which is what makes any of this
    /// checkable at all.
    #[test]
    fn the_same_seed_is_the_same_sound() {
        assert_eq!(brown_noise(2_000, 3), brown_noise(2_000, 3));
        assert_ne!(brown_noise(2_000, 3), brown_noise(2_000, 4));
    }

    /// Nothing may sit at a steady offset from zero: a signal that does is a
    /// speaker cone pushed permanently out of place.
    #[test]
    fn nothing_carries_a_steady_offset() {
        let brown = brown_noise(48_000, 17);
        let mean = brown.iter().sum::<f32>() / brown.len() as f32;
        assert!(mean.abs() < 0.15, "the bed sits at {mean} rather than zero");
    }
}
