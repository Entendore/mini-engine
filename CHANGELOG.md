# Changelog

## 0.5.0 — UI, actions, tweens, persistence, sprites
- Immediate-mode `Ui` (button/checkbox/slider/panel/label/progress) with events.
- `ActionMap` named bindings; `Input::mouse_delta`; `KeyCode::Backspace`.
- Audio buses (`Bus::Music`/`Sfx`), per-bus volume, `stop_all`.
- `SpriteImage`/`AnimatedSprite` + `animate_sprites`/`draw_sprites`; atlas frames Rc-cached.
- `Tween` primitive + `Tweens` manager (`FnMut(&mut World, f32)`).
- `Storage` trait (`ctx.storage`): `MemoryStorage` in core, `FileStorage` in facade.
- `physics::{collision_pairs, overlaps}`; `RecordingBackend` for deterministic replays.
- Fix: `Pool::insert` rejects stale handles even into empty retired slots.

## 0.4.0 — assets, scenes, interpolation, audio control
- `Image` + `Renderer::blit(_opts)`: alpha-blended, tintable, flippable blits.
- `Atlas` (grid frames) + `Animator`; PNG loading in `mini-engine` (`png` feature).
- `Scene`/`SceneStack`/`Transition`: optional scene management that implements `Game`.
- Render interpolation: `Time::alpha()`, `Transform::{prev, advance, snap, interp}`.
- Audio: looping sounds, `VoiceHandle` (`stop`/`is_active`).
- Input: typed chars (`Input::typed`), `Num0..=Num9`; `Timer`; `math::ease`.

## 0.3.0 — future-proofing
- Workspace split: `mini-core` (OS-free, dependency-free) + `mini-engine` (facade).
- Host contract `WindowBackend`; built-in `ScriptedBackend` enables headless tests.
- ECS: arbitrary user components via a TypeId registry.
- Stability policy: `#[non_exhaustive]` enums, `EngineConfig` builder, `Sound` presets.

## 0.2.0 — correctness + capability
- Generational entity handles; mouse input; camera on the renderer; real audio (cpal).
- `Ctx::quit()`; pause when unfocused; config validation.

## 0.1.0 — first working engine
- Fixed-timestep loop, minifb window, software framebuffer, bitmap font,
  keyboard input with edge detection, xorshift RNG, minimal ECS, Pong demo.
