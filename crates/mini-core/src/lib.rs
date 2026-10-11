//! # mini-core
//!
//! The OS-free heart of the engine: no display, no audio device, no
//! third-party crates. Portable to desktop, `wasm32`, CI — and gameplay is
//! testable in plain `cargo test` via [`backend::ScriptedBackend`].
//!
//! Layering, outside-in:
//! 1. [`backend`] — the host contract ([`backend::WindowBackend`],
//!    [`backend::RecordingBackend`]).
//! 2. [`engine`] — the [`engine::Game`] trait and the fixed-timestep loop.
//! 3. [`scene`] — optional scene management ([`scene::SceneStack`] *is* a `Game`).
//! 4. [`ui`] — immediate-mode widgets; [`tween`] — animation;
//!    [`physics`] — AABB queries; [`storage`] — persistence contract.
//! 5. [`ecs`], [`render`], [`audio`], [`input`], [`math`], [`time`], [`rng`].
//!
//! ## Stability policy
//! - Semver: while `0.x`, breaking changes bump the minor version.
//! - New [`engine::Game`] / [`scene::Scene`] methods always ship defaulted.
//! - Enums that may grow are `#[non_exhaustive]` — match with a wildcard arm.
//! - Construct option structs with struct-update syntax
//!   (`Sound { freq_hz: 880.0, ..Sound::BEEP }`) so added fields never break you.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod audio;
pub mod backend;
pub mod config;
pub mod ecs;
pub mod engine;
pub mod error;
pub mod input;
pub mod math;
pub mod physics;
pub mod render;
pub mod rng;
pub mod scene;
pub mod storage;
pub mod time;
pub mod tween;
pub mod ui;

pub use error::{EngineError, Result};

/// Everything a game typically needs, in one `use`.
pub mod prelude {
    pub use crate::audio::{Audio, Bus, Mixer, Sound, VoiceHandle, Waveform};
    pub use crate::backend::{FrameInput, RecordingBackend, ScriptedBackend, WindowBackend};
    pub use crate::config::EngineConfig;
    pub use crate::ecs::{
        join2, join2_mut, join3, join3_mut, Collider, Component, Entity, Lifetime, Pool, Sprite,
        Transform, Velocity, World,
    };
    pub use crate::engine::{Ctx, Engine, Game};
    pub use crate::input::{ActionMap, Input, KeyCode, MouseButton};
    pub use crate::math::{ease, Rect, Vec2};
    pub use crate::physics::{collision_pairs, overlaps};
    pub use crate::render::{
        animate_sprites, draw_sprites, AnimatedSprite, Animator, Atlas, Blit, Color, Image,
        Renderer, SpriteImage,
    };
    pub use crate::rng::Rng;
    pub use crate::scene::{Scene, SceneStack, Transition};
    pub use crate::storage::{MemoryStorage, Storage};
    pub use crate::time::{Time, Timer};
    pub use crate::tween::{EaseFn, Tween, TweenId, Tweens};
    pub use crate::ui::{Ui, UiEvent, WidgetId};
    pub use crate::Result;
}
