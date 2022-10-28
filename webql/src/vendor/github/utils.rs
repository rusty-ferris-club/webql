use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// convert [`Value`] string data to [`DateTime<Utc>`]
pub fn parse_to_date_time(v: &Value) -> Result<DateTime<Utc>> {
    match v
        .as_str()
        .with_context(|| format!("could not convert date: {:?} to as_str", v))?
        .parse()
    {
        Ok(dt) => Ok(dt),
        Err(e) => bail!(e),
    }
}
