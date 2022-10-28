//! Vendors implementation for fetching data and run filters on the JSON
//! response. The list of vendors is enabled bt feature flag on
#[cfg(feature = "github")]
pub mod github;
