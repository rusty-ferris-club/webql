//! Public structs
use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use serde_json::Value;

/// Describe the data kind that fetched from the one of the vendors.
#[derive(Debug, Clone)]
pub enum EventKind {
    #[cfg(feature = "github")]
    PR,
    #[cfg(feature = "github")]
    PrComment,
    #[cfg(feature = "github")]
    PrEvent,
}

/// Describe the event details that return from the vendors.
#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub id: String,
    pub parent_event_id: Option<String>,
    pub name: String,
    pub link: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub priority: usize,
    pub row_data: Value, // pub status: String,
}

/// Operation type on the JSON value
#[derive(Debug, Deserialize, Clone)]
pub enum Operation {
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "~")]
    Contains,
}

/// Filter options
#[derive(Debug, Deserialize, Clone)]
pub struct Filter {
    pub query: String,
    pub values: Vec<String>,
    pub operation: Operation,
}
