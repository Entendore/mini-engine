use crate::render::Color;

/// Startup settings. Built with chained methods; every knob has a default.
///
/// ```ignore
/// let config = EngineConfig::new()
///     .title("PONG")
///     .size(640, 360)
///     .update_hz(120.0);
/// ```
#[derive(Debug, Clone)]
pub struct EngineConfig {
    title: String,
    width: usize,
    height: usize,
    target_fps: u32,
    update_hz: f32,
    quit_on_escape: bool,
    pause_when_unfocused: bool,
    enable_audio: bool,
    clear_color: Color,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "mini engine".into(),
            width: 640,
            height: 360,
            target_fps: 60,
            update_hz: 60.0,
            quit_on_escape: true,
            pause_when_unfocused: true,
            enable_audio: true,
            clear_color: Color::BLACK,
        }
    }
}

impl EngineConfig {
    /// Defaults: 640x360, 60 fps render, 60 Hz update.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    pub fn update_hz(mut self, hz: f32) -> Self {
        self.update_hz = hz;
        self
    }

    pub fn quit_on_escape(mut self, yes: bool) -> Self {
        self.quit_on_escape = yes;
        self
    }

    pub fn pause_when_unfocused(mut self, yes: bool) -> Self {
        self.pause_when_unfocused = yes;
        self
    }

    pub fn enable_audio(mut self, yes: bool) -> Self {
        self.enable_audio = yes;
        self
    }

    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    // -- getters ------------------------------------------------------------

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn update_hz(&self) -> f32 {
        self.update_hz
    }

    pub fn quit_on_escape(&self) -> bool {
        self.quit_on_escape
    }

    pub fn pause_when_unfocused(&self) -> bool {
        self.pause_when_unfocused
    }

    pub fn enable_audio(&self) -> bool {
        self.enable_audio
    }

    pub fn clear_color(&self) -> Color {
        self.clear_color
    }
}
