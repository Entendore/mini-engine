//! # mini-engine
//!
//! Desktop facade over [`mini_core`]: a real window (minifb), PNG assets
//! (cargo feature `png`), OS audio output (cargo feature `audio`), and
//! file-backed persistence.
//!
//! ```ignore
//! use mini_engine::prelude::*;
//! fn main() -> Result<()> {
//!     run(EngineConfig::new().title("hi").size(480, 270), &mut MyGame)
//! }
//! ```
//!
//! Need something else? Depend on `mini_core` directly and drive
//! [`core::engine::Engine`] with your own [`core::backend::WindowBackend`] —
//! that is all this crate does.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod assets;
pub mod backend;
pub mod storage;

#[cfg(feature = "audio")]
mod audio_output;

/// The OS-free engine, namespaced for direct access.
pub use mini_core as core;

/// Re-export the core's game-facing API at the crate root.
pub use mini_core::prelude::*;

/// Root + prelude exports, including [`run`].
pub mod prelude {
    pub use mini_core::prelude::*;
    pub use crate::run;
}

/// Boot the engine with the default desktop backends and run `game` until
/// the window closes, Escape is pressed (if enabled), or the game calls
/// [`Ctx::quit`](mini_core::engine::Ctx::quit).
///
/// Degradations are graceful: no audio device → silence; no data directory →
/// in-memory saves. Only a missing window aborts the launch.
pub fn run<G: Game>(config: EngineConfig, game: &mut G) -> Result<()> {
    let mut engine = core::Engine::new(config.clone())?;

    // Persistence: file-backed under the app's OS data dir, if possible.
    match storage::FileStorage::in_app_dir(config.title()) {
        Ok(fs) => engine.set_storage(Box::new(fs)),
        Err(e) => {
            eprintln!("mini_engine: persistence unavailable ({e}); saves stay in memory")
        }
    }

    // Audio: open the default output device and let it pull from the mixer.
    #[cfg(feature = "audio")]
    let _stream = if config.enable_audio() {
        match audio_output::try_start(engine.audio().mixer()) {
            Ok(stream) => Some(stream),
            Err(e) => {
                eprintln!("mini_engine: audio unavailable ({e}); running silent");
                None
            }
        }
    } else {
        None
    };

    // Window + input + frame pacing.
    let mut host = backend::MinifbBackend::new(&config)?;
    engine.run(&mut host, game)
}
