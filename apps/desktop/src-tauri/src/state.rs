use std::sync::{Mutex, MutexGuard};

use halo_store::Connection;

use crate::error::{Error, Result};
use crate::timer::LiveTimer;

/// Shared app state, managed by Tauri.
pub struct AppState {
    db: Mutex<Connection>,
    timer: Mutex<LiveTimer>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self {
            db: Mutex::new(db),
            timer: Mutex::new(LiveTimer::new()),
        }
    }

    pub fn db(&self) -> Result<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| Error::Poisoned)
    }

    /// Lock order: take `timer` before `db` when holding both.
    pub fn timer(&self) -> Result<MutexGuard<'_, LiveTimer>> {
        self.timer.lock().map_err(|_| Error::Poisoned)
    }
}
