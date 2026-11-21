use std::sync::{Mutex, MutexGuard};

use halo_store::Connection;

use crate::error::{Error, Result};

/// Shared app state, managed by Tauri.
pub struct AppState {
    db: Mutex<Connection>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self { db: Mutex::new(db) }
    }

    pub fn db(&self) -> Result<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| Error::Poisoned)
    }
}
