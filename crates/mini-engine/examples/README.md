# Examples gallery

Read top to bottom — each example needs only the ones before it.
Every file is a complete, runnable game; the comments are the docs.

| #  | example            | teaches                                              | run with                                                       |
|----|--------------------|------------------------------------------------------|----------------------------------------------------------------|
| 01 | `01_hello_window`  | `Game`, `run()`, `Ctx`, text                          | `cargo run --example 01_hello_window`                          |
| 02 | `02_input`         | held/pressed/released, mouse, wheel, typed text       | `cargo run --example 02_input`                                 |
| 03 | `03_drawing`       | primitives, camera, blit tint/alpha/flip              | `cargo run --example 03_drawing`                               |
| 04 | `04_game_loop`     | fixed timestep, `time.alpha()` interpolation          | `cargo run --example 04_game_loop`                             |
| 05 | `05_ecs`           | spawn / custom components / systems / collision       | `cargo run --example 05_ecs`                                   |
| 06 | `06_audio`         | `Sound`, waveforms, buses, `VoiceHandle`              | `cargo run --example 06_audio --features audio`                |
| 07 | `07_scenes`        | `SceneStack`, `Transition`, shared state              | `cargo run --example 07_scenes`                                |
| 08 | `08_ui`            | `Ui` button/slider/checkbox/progress/events           | `cargo run --example 08_ui`                                    |
| 09 | `09_tweens`        | `Tween`, `Tweens`, easing functions                   | `cargo run --example 09_tweens`                                |
| 10 | `10_storage`       | `ctx.storage` save/load/delete                        | `cargo run --example 10_storage`                               |
| 11 | `11_atlas_anim`    | `Atlas`, `Animator`, `AnimatedSprite`                 | `cargo run --example 11_atlas_anim`                            |
| 12 | `12_headless`      | `ScriptedBackend`, `RecordingBackend`, CI simulation  | `cargo run --example 12_headless` (no window!)                 |
| 13 | `13_platformer`    | actions, tile collision, camera, pickups              | `cargo run --example 13_platformer`                            |
| -- | `pong`             | capstone: everything together                         | `cargo run --example pong`                                     |

Notes:
- Without `--features audio` the engine mixes silently; every example still runs.
- With `--features png`, `mini_engine::assets::load_png("file.png")` decodes
  images for `blit` / `Atlas::new` (see the comment in 11).
