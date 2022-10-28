//! Filter Json value with filters
//!
//!
//! # Example:
//! ```
#![doc = include_str!("../examples/json-filter.rs")]
//! ```
//!
use anyhow::{bail, Result};
use serde_json::Value;
use tracing::debug;

use super::data::{Filter, Operation};

/// Filter json [`Value`] object with the [`Filter`] settings
///
/// # Arguments
/// * `data` - Event data
/// * `filters` - List of filter queries
///
/// # Errors
/// - When [`Filter`] query is invalid
pub fn is_match_filters(data: &Value, filters: &[Filter]) -> Result<bool> {
    for filter in filters {
        let query_result = match jql::walker(data, &filter.query) {
            Ok(q) => q,
            Err(e) => {
                debug!(message = "could not run jql walker", query = filter.query);
                bail!("{}", e)
            }
        };

        // allow single_match_else for now to support more type cases.
        #[allow(clippy::single_match_else)]
        let is_match = match &query_result {
            // check query value type for different logic
            Value::Array(v) => is_match_array(v, filter),
            // Default meaning is string value
            _ => {
                let event_value = query_result.as_str().unwrap_or("");
                if event_value.is_empty() {
                    debug!(message = "value is empty", query = filter.query);
                    bail!("query {} result is empty", filter.query);
                }
                debug!(
                    message = "found value from pull request data",
                    value = event_value,
                    query = filter.query,
                );
                is_match_string(event_value, filter)
            }
        };

        if !is_match {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Chec
///
/// # Arguments
/// * `val_str` - Filter value
/// * `filter` - Group filters
fn is_match_string(val_str: &str, filter: &Filter) -> bool {
    match filter.operation {
        Operation::Equal => {
            debug!(
                message = "check equal value",
                group_values = format!("{:?}", filter.values),
                value = val_str,
                operation = "equal",
            );
            filter.values.contains(&val_str.to_string())
        }
        Operation::Contains => {
            let mut exit = false;
            for group_val in &filter.values {
                debug!(
                    message = "check contains values",
                    group_value = group_val,
                    value = val_str,
                    operation = "contains",
                );
                if val_str.contains(group_val) {
                    exit = true;
                    break;
                }
            }
            exit
        }
    }
}

/// Run group filters on a array
///
/// # Arguments
/// * `values` - List of values
/// * `filter` - Group filters
fn is_match_array(values: &Vec<Value>, filter: &Filter) -> bool {
    for value in values {
        let pr_value = value.as_str().unwrap_or("");
        if is_match_string(pr_value, filter) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test_jfilter {

    use insta::assert_debug_snapshot;
    use serde_json::json;

    use super::{Filter, Operation, Value};
    use crate::jfilter::{is_match_array, is_match_filters, is_match_string};

    #[test]
    fn is_equal_match_string() {
        let filter = Filter {
            query: "".to_string(),
            values: vec!["foo".to_string(), "exists-value".to_string()],
            operation: Operation::Equal,
        };
        assert_debug_snapshot!(is_match_string("exists-value", &filter));
        assert_debug_snapshot!(is_match_string("equal-value", &filter));
    }

    #[test]
    fn is_contains_match_string() {
        let filter = Filter {
            query: "".to_string(),
            values: vec!["foo".to_string(), "exists-value".to_string()],
            operation: Operation::Contains,
        };
        assert_debug_snapshot!(is_match_string("exists-value", &filter));
        assert_debug_snapshot!(is_match_string("contains-value", &filter));
    }

    #[test]
    fn is_contains_match_array() {
        let filter = Filter {
            query: "".to_string(),
            values: vec!["foo".to_string(), "contains".to_string()],
            operation: Operation::Contains,
        };
        assert_debug_snapshot!(is_match_array(
            &vec![
                Value::String("val".to_string()),
                Value::String("value-contains".to_string())
            ],
            &filter
        ));
        assert_debug_snapshot!(is_match_array(
            &vec![
                Value::String("val".to_string()),
                Value::String("val-2".to_string())
            ],
            &filter
        ));
    }

    #[test]
    fn can_match_filters() {
        let json = json!({
            "body": "some example",
            "labels": [
                {
                    "name": "label-1",
                },
                {
                    "name": "label-2",
                },
            ],
            "user" : {
                "login": "kaplanelad"
            }
        });
        let filter = vec![
            Filter {
                query: r#""body""#.to_string(),
                values: vec!["foo".to_string(), "example".to_string()],
                operation: Operation::Contains,
            },
            Filter {
                query: r#""user"."login""#.to_string(),
                values: vec!["foo".to_string(), "kaplanelad".to_string()],
                operation: Operation::Equal,
            },
            Filter {
                query: r#""labels"|={"name"}."name""#.to_string(),
                values: vec!["foo".to_string(), "label-1".to_string()],
                operation: Operation::Equal,
            },
            Filter {
                query: r#""labels"|={"name"}."name""#.to_string(),
                values: vec!["foo".to_string(), "label".to_string()],
                operation: Operation::Contains,
            },
        ];
        assert_debug_snapshot!(is_match_filters(&json, &filter));
    }
}
