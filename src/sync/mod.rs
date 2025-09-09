#[allow(clippy::module_inception)]
pub mod sync;

// re-export for a flat public API: rkik::sync::{sync_from_probe, SyncError}
pub use sync::{SyncError, sync_from_probe};
