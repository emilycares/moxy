//! This contains the logic to modify the configuration an filesystem inorder to support
//! `"build_mode": "write"`

/// This contains the main builder functionality. That is called by the router.
pub mod core;
/// This contains the main builder functionality for ws.
pub mod ws;
/// This contains the logic off feching new data.
pub mod request;
/// This contains how new data is saved.
pub mod storage;
