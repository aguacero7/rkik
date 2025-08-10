#[derive(Debug)]
pub enum RkikError {
    ResolveError(String),
    SyncError(String),
    General(String),
}

pub type Result<T> = std::result::Result<T, RkikError>;
