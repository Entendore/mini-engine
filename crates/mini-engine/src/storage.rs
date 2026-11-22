//! File-backed [`Storage`](mini_core::storage::Storage) in a per-application
//! data directory.

use std::io::Read;
use std::path::PathBuf;

use mini_core::error::{EngineError, Result};
use mini_core::storage::Storage;

/// Stores each key as a text file under a per-app directory
/// (`$XDG_DATA_HOME/<app>` or `~/.local/share/<app>` on Unix,
/// `%APPDATA%\<app>` on Windows). Keys and the app name are sanitized to
/// `[A-Za-z0-9_-]`.
pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    /// A store rooted at the OS data dir for `app` (usually the window title).
    pub fn in_app_dir(app: &str) -> Result<Self> {
        let base = if cfg!(windows) {
            std::env::var_os("APPDATA").map(PathBuf::from)
        } else {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        }
        .ok_or_else(|| EngineError::Storage("no home/data directory found".into()))?;

        let dir = base.join(sanitize(app));
        std::fs::create_dir_all(&dir).map_err(|e| EngineError::Storage(e.to_string()))?;
        Ok(Self { dir })
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(sanitize(key))
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

impl Storage for FileStorage {
    fn save(&mut self, key: &str, value: &str) -> Result<()> {
        std::fs::write(self.path(key), value.as_bytes())
            .map_err(|e| EngineError::Storage(e.to_string()))
    }

    fn load(&self, key: &str) -> Option<String> {
        let mut out = String::new();
        std::fs::File::open(self.path(key)).ok()?.read_to_string(&mut out).ok()?;
        Some(out)
    }

    fn delete(&mut self, key: &str) {
        let _ = std::fs::remove_file(self.path(key));
    }
}
