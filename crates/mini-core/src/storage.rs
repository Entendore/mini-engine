//! Persistence seam. The core defines the contract and an in-memory impl;
//! hosts provide real backends (`mini-engine` ships a file-backed one).
//!
//! Values are strings on purpose: games pick their own encoding (plain
//! numbers, JSON, `bincode`...). A tiny trait means it can be backed by
//! files, wasm `localStorage`, or a network service without churn.

use std::collections::HashMap;

/// Key/value persistence for saves, settings, high scores.
pub trait Storage {
    /// Store `value` under `key`, replacing any previous value.
    fn save(&mut self, key: &str, value: &str) -> crate::Result<()>;
    /// Load the value stored under `key`, if any.
    fn load(&self, key: &str) -> Option<String>;
    /// Remove `key` (no-op if absent).
    fn delete(&mut self, key: &str);
}

/// Ephemeral backend: a `HashMap`. The engine default; what tests use.
#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    map: HashMap<String, String>,
}

impl MemoryStorage {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn save(&mut self, key: &str, value: &str) -> crate::Result<()> {
        self.map.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    fn load(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    fn delete(&mut self, key: &str) {
        self.map.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_delete() {
        let mut s = MemoryStorage::new();
        assert_eq!(s.load("missing"), None);
        s.save("best", "42").unwrap();
        assert_eq!(s.load("best").as_deref(), Some("42"));
        s.save("best", "43").unwrap(); // overwrite
        assert_eq!(s.load("best").as_deref(), Some("43"));
        s.delete("best");
        assert_eq!(s.load("best"), None);
        s.delete("best"); // idempotent
    }
}
