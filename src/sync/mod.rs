// build only when the "sync" feature is enabled
#![cfg(feature = "sync")]

pub mod sync;

// re-export for a flat public API: rkik::sync::{sync_from_probe, SyncError}
pub use sync::{sync_from_probe, SyncError};
