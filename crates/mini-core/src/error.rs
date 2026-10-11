use std::fmt;

/// Errors the engine can surface to the host application.
///
/// `#[non_exhaustive]`: new variants may appear in minor releases — match
/// with a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum EngineError {
    /// Startup settings failed validation.
    InvalidConfig(String),
    /// The host could not create its surface.
    WindowInit(String),
    /// A frame could not be presented.
    Present(String),
    /// The audio output device could not be opened.
    AudioInit(String),
    /// An asset failed to load or decode.
    Asset(String),
    /// A storage backend failed to read or write.
    Storage(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            EngineError::WindowInit(m) => write!(f, "window init failed: {m}"),
            EngineError::Present(m) => write!(f, "frame present failed: {m}"),
            EngineError::AudioInit(m) => write!(f, "audio init failed: {m}"),
            EngineError::Asset(m) => write!(f, "asset error: {m}"),
            EngineError::Storage(m) => write!(f, "storage error: {m}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type Result<T> = std::result::Result<T, EngineError>;
