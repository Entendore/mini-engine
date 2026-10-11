//! Audio — the runtime synth. No sound files anywhere.
//!
//!   1-8        play a scale (hold SHIFT for square wave)
//!   SPACE      laser (pitch slide)
//!   ENTER      toggle a looping drone (VoiceHandle)
//!   M          cycle music-bus volume   O/P  master down/up
//!
//! Run with `--features audio` to hear it. Without the feature the engine
//! still mixes silently, so this example works everywhere.

use mini_engine::prelude::*;

const SCALE_KEYS: [KeyCode; 8] = [
    KeyCode::Num1,
    KeyCode::Num2,
    KeyCode::Num3,
    KeyCode::Num4,
    KeyCode::Num5,
    KeyCode::Num6,
    KeyCode::Num7,
    KeyCode::Num8,
];
const RATIOS: [f32; 8] =
    [1.0, 9.0 / 8.0, 5.0 / 4.0, 4.0 / 3.0, 3.0 / 2.0, 5.0 / 3.0, 15.0 / 8.0, 2.0];
const BASE: f32 = 261.626; // middle C

struct Piano {
    drone: Option<VoiceHandle>,
    music_vol: f32,
    master: f32,
}

impl Game for Piano {
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.audio.set_bus_volume(Bus::Music, self.music_vol);
        ctx.audio.set_master_volume(self.master);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        // Scale: same notes, different oscillator if SHIFT is held.
        for (i, key) in SCALE_KEYS.iter().enumerate() {
            if ctx.input.pressed(*key) {
                ctx.audio.play(Sound {
                    freq_hz: BASE * RATIOS[i],
                    wave: if ctx.input.held(KeyCode::LeftShift) {
                        Waveform::Square
                    } else {
                        Waveform::Sine
                    },
                    ms: 350,
                    volume: 0.6,
                    ..Sound::BEEP
                });
            }
        }

        // A pitch slide makes a laser: `slide_hz` is added across the sound.
        if ctx.input.pressed(KeyCode::Space) {
            ctx.audio.play(Sound {
                freq_hz: 900.0,
                slide_hz: -700.0,
                ms: 300,
                wave: Waveform::Square,
                ..Sound::BEEP
            });
        }

        // Looping drone on the Music bus; the handle stops it on demand.
        if ctx.input.pressed(KeyCode::Enter) {
            if let Some(d) = self.drone.take() {
                d.stop();
            } else {
                self.drone = Some(ctx.audio.play(Sound {
                    freq_hz: 110.0,
                    ms: 800,
                    wave: Waveform::Triangle,
                    volume: 0.5,
                    bus: Bus::Music,
                    loops: true,
                    ..Sound::BEEP
                }));
            }
        }

        if ctx.input.pressed(KeyCode::M) {
            self.music_vol = if self.music_vol >= 1.0 { 0.0 } else { self.music_vol + 0.25 };
            ctx.audio.set_bus_volume(Bus::Music, self.music_vol);
        }
        if ctx.input.pressed(KeyCode::O) {
            self.master = (self.master - 0.1).max(0.0);
            ctx.audio.set_master_volume(self.master);
        }
        if ctx.input.pressed(KeyCode::P) {
            self.master = (self.master + 0.1).min(1.0);
            ctx.audio.set_master_volume(self.master);
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(14, 16, 26));

        // Draw the 8 keys; held keys light up.
        for (i, key) in SCALE_KEYS.iter().enumerate() {
            let rect = Rect::new(40.0 + i as f32 * 50.0, 150.0, 40.0, 70.0);
            let lit = ctx.input.held(*key);
            r.rect(rect, if lit { Color::YELLOW } else { Color::rgb(38, 44, 66) });
            r.rect_outline(rect, Color::GRAY);
            r.text(rect.x as i32 + 16, rect.y as i32 + 28, &format!("{}", i + 1), 2, Color::BLACK);
        }

        r.text_centered(60, "AUDIO SYNTH - NO FILES, ALL RUNTIME", 3, Color::WHITE);
        r.text_centered(95, "1-8 NOTES   SHIFT = SQUARE   SPACE = LASER", 2, Color::GRAY);

        // Drone + volume readouts.
        let drone = if self.drone.as_ref().is_some_and(|d| d.is_active()) { "ON " } else { "OFF" };
        r.text(40, 240, &format!("DRONE {drone}  (ENTER)"), 2, Color::CYAN);
        r.rect(Rect::new(200.0, 240.0, 80.0 * self.music_vol, 8.0), Color::GREEN);
        r.text(290, 236, "MUSIC", 2, Color::GRAY);
        r.rect(Rect::new(360.0, 240.0, 80.0 * self.master, 8.0), Color::ORANGE);
        r.text(450, 236, "MASTER", 2, Color::GRAY);
        r.text_right(472, 8, &format!("VOICES {}", ctx.audio.active_voices()), 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game = Piano { drone: None, music_vol: 0.6, master: 1.0 };
    run(EngineConfig::new().title("06 - audio").size(480, 270), &mut game)
}
