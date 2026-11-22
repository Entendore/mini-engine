//! Audio concern, in two small layers:
//!
//! 1. [`Sound`] — a fire-and-forget request. Build on the [`Sound::BEEP`]
//!    preset with struct-update syntax so new fields can't break call sites.
//! 2. [`Mixer`] — the synth: a voice bank with envelopes, per-bus volumes,
//!    and a soft-clipping master bus. Pure DSP, unit-testable, no I/O.
//!
//! [`Audio`] hands the mixer out as an `Arc<Mutex<…>>`; *any* consumer can
//! pull samples — the cpal stream in `mini-engine`, a WAV file writer, a
//! network stream. Output is a consumer, not a trait the core must define.

use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

/// Oscillator shape for a [`Sound`]. `#[non_exhaustive]`: match with a
/// wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Waveform {
    /// Pure tone.
    Sine,
    /// Buzzy, chip-tune flavor.
    Square,
    /// Softer buzz.
    Triangle,
    /// White-noise-ish (xorshift).
    Noise,
}

/// Mix bus. Each bus has its own volume; the master applies on top of both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Bus {
    /// Looping music, ambience.
    Music,
    /// One-shot effects.
    Sfx,
}

/// A one-shot (or looping) sound request.
#[derive(Debug, Clone, Copy)]
pub struct Sound {
    /// Base frequency in Hz.
    pub freq_hz: f32,
    /// Linear pitch slide: Hz added to `freq_hz` by the end of each cycle.
    pub slide_hz: f32,
    /// Length in milliseconds (ignored while looping — cycles forever).
    pub ms: u32,
    pub wave: Waveform,
    /// Per-sound loudness in `0.0..=1.0` (before bus and master volume).
    pub volume: f32,
    /// When true, the sound repeats until its [`VoiceHandle`] is stopped.
    pub loops: bool,
    /// Which [`Bus`] mixes this sound.
    pub bus: Bus,
}

impl Sound {
    /// Neutral beep other sounds derive from:
    /// `Sound { freq_hz: 880.0, wave: Waveform::Square, ..Sound::BEEP }`.
    /// Adding a field to `Sound` cannot break this pattern.
    pub const BEEP: Sound = Sound {
        freq_hz: 440.0,
        slide_hz: 0.0,
        ms: 80,
        wave: Waveform::Sine,
        volume: 0.5,
        loops: false,
        bus: Bus::Sfx,
    };
}

/// Identifies a voice inside a [`Mixer`]. Opaque; obtained from `play`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceId(u64);

struct Voice {
    id: VoiceId,
    sound: Sound,
    age: f32,
    phase: f32,
    noise: u32,
}

/// Voice bank + bus volumes + master bus. Push sounds in, pull samples out.
pub struct Mixer {
    sample_rate: f32,
    master: f32,
    music_vol: f32,
    sfx_vol: f32,
    voices: Vec<Voice>,
    cap: usize,
    next_id: u64,
}

impl Mixer {
    /// A mixer at `sample_rate` Hz (match your output device once, up front).
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            master: 0.6,
            music_vol: 1.0,
            sfx_vol: 1.0,
            voices: Vec::new(),
            cap: 64,
            next_id: 1,
        }
    }

    /// Master loudness in `0.0..=1.0`.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master = volume.clamp(0.0, 1.0);
    }

    /// Set one bus's volume (0.0..=1.0); master applies on top.
    pub fn set_bus_volume(&mut self, bus: Bus, volume: f32) {
        let v = volume.clamp(0.0, 1.0);
        match bus {
            Bus::Music => self.music_vol = v,
            Bus::Sfx => self.sfx_vol = v,
        }
    }

    /// Match the output device's rate (call once, before playback starts).
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if sample_rate > 0.0 {
            self.sample_rate = sample_rate;
        }
    }

    pub fn active_voices(&self) -> usize {
        self.voices.len()
    }

    /// Queue a sound; returns its handle id. Full bank → oldest voice cut.
    /// Zero-length or silent sounds return an id that is simply inactive.
    pub fn play(&mut self, sound: Sound) -> VoiceId {
        let id = VoiceId(self.next_id);
        self.next_id += 1;
        if sound.ms == 0 || sound.volume <= 0.0 {
            return id;
        }
        if self.voices.len() >= self.cap {
            self.voices.remove(0);
        }
        self.voices.push(Voice { id, sound, age: 0.0, phase: 0.0, noise: 0x9E37_79B9 });
        id
    }

    /// Silence every voice with this id.
    pub fn stop(&mut self, id: VoiceId) {
        self.voices.retain(|v| v.id != id);
    }

    /// Silence everything immediately (scene changes, pause menus...).
    pub fn stop_all(&mut self) {
        self.voices.clear();
    }

    pub fn is_active(&self, id: VoiceId) -> bool {
        self.voices.iter().any(|v| v.id == id)
    }

    /// Fill `out` (interleaved, `channels` samples per frame) with the mix.
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        let dt = 1.0 / self.sample_rate.max(1.0);
        let channels = channels.max(1);
        for frame in out.chunks_mut(channels) {
            let mut sample = 0.0;
            let mut i = 0;
            while i < self.voices.len() {
                let v = &mut self.voices[i];
                let dur = v.sound.ms as f32 / 1000.0;
                v.age += dt;
                if !v.sound.loops && v.age >= dur {
                    self.voices.swap_remove(i);
                    continue;
                }
                let progress = if v.sound.loops {
                    (v.age % dur) / dur
                } else {
                    v.age / dur
                };
                let freq = v.sound.freq_hz + v.sound.slide_hz * progress;
                v.phase = (v.phase + TAU * freq * dt) % TAU;

                // Short attack (kills clicks) + quadratic release per cycle.
                let attack = (v.age / 0.004).min(1.0);
                let env = attack * (1.0 - progress).powi(2);

                let raw = match v.sound.wave {
                    Waveform::Sine => v.phase.sin(),
                    Waveform::Square => if v.phase.sin() >= 0.0 { 1.0 } else { -1.0 },
                    Waveform::Triangle => 1.0 - 4.0 * ((v.phase / TAU) - 0.5).abs(),
                    Waveform::Noise => {
                        v.noise ^= v.noise << 13;
                        v.noise ^= v.noise >> 17;
                        v.noise ^= v.noise << 5;
                        (v.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
                    }
                };
                let bus_vol = match v.sound.bus {
                    Bus::Music => self.music_vol,
                    Bus::Sfx => self.sfx_vol,
                };
                sample += raw * env * v.sound.volume * bus_vol;
                i += 1;
            }
            // tanh acts as a gentle limiter when many voices pile up.
            sample = (sample * self.master).tanh();
            for ch in frame {
                *ch = sample;
            }
        }
    }
}

/// Handle the game receives in `Ctx`. Cheap to clone, thread-safe underneath.
#[derive(Clone)]
pub struct Audio {
    mixer: Arc<Mutex<Mixer>>,
}

/// Control token for a queued sound: stop it or poll whether it lives.
#[derive(Debug, Clone)]
pub struct VoiceHandle {
    mixer: Arc<Mutex<Mixer>>,
    id: VoiceId,
}

impl VoiceHandle {
    /// Silence this voice (no-op if it already ended).
    pub fn stop(&self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.stop(self.id);
        }
    }

    /// Still audible?
    pub fn is_active(&self) -> bool {
        self.mixer.lock().map(|m| m.is_active(self.id)).unwrap_or(false)
    }
}

impl Audio {
    pub(crate) fn new() -> Self {
        Self { mixer: Arc::new(Mutex::new(Mixer::new(48_000.0))) }
    }

    /// Queue a sound and get a handle for stopping/polling it.
    pub fn play(&self, sound: Sound) -> VoiceHandle {
        let id = self
            .mixer
            .lock()
            .map(|mut m| m.play(sound))
            .unwrap_or(VoiceId(0));
        VoiceHandle { mixer: Arc::clone(&self.mixer), id }
    }

    pub fn set_master_volume(&self, volume: f32) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.set_master_volume(volume);
        }
    }

    pub fn set_bus_volume(&self, bus: Bus, volume: f32) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.set_bus_volume(bus, volume);
        }
    }

    /// Silence every voice (scene transitions, pause menus...).
    pub fn stop_all(&self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.stop_all();
        }
    }

    pub fn active_voices(&self) -> usize {
        self.mixer.lock().map(|m| m.active_voices()).unwrap_or(0)
    }

    /// Escape hatch for custom output pipelines (file rendering, etc.).
    pub fn mixer(&self) -> Arc<Mutex<Mixer>> {
        Arc::clone(&self.mixer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shots_expire_loops_persist() {
        let mut mixer = Mixer::new(1_000.0);
        let id = mixer.play(Sound { ms: 10, loops: true, ..Sound::BEEP });
        let mut buf = [0.0f32; 100]; // ten cycles' worth of time
        mixer.render(&mut buf, 1);
        assert!(mixer.is_active(id), "looping voice survives its cycle length");
        assert!(buf.iter().any(|s| s.abs() > 0.0));

        mixer.stop(id);
        assert!(!mixer.is_active(id));
        mixer.render(&mut buf, 1);
        assert!(buf.iter().all(|s| *s == 0.0), "stopped voice is silent");
    }

    #[test]
    fn one_shot_expires() {
        let mut mixer = Mixer::new(1_000.0);
        let id = mixer.play(Sound { ms: 10, loops: false, ..Sound::BEEP });
        let mut buf = [0.0f32; 40]; // 40 ms at 1 kHz; the sound lasts 10 ms
        mixer.render(&mut buf, 1);
        assert!(!mixer.is_active(id));
    }

    #[test]
    fn buses_mix_independently() {
        let mut mixer = Mixer::new(1_000.0);
        mixer.set_bus_volume(Bus::Music, 0.0);
        mixer.play(Sound { bus: Bus::Music, ..Sound::BEEP });
        let mut buf = [0.0f32; 20];
        mixer.render(&mut buf, 1);
        assert!(buf.iter().all(|s| *s == 0.0), "muted music bus is silent");

        mixer.set_bus_volume(Bus::Music, 1.0);
        mixer.render(&mut buf, 1);
        assert!(buf.iter().any(|s| s.abs() > 0.0));

        mixer.stop_all();
        mixer.render(&mut buf, 1);
        assert!(buf.iter().all(|s| *s == 0.0));
    }
}
