use serde_json::json;
use webql::{
    data::{Filter, Operation},
    jfilter,
};

fn main() {
    let json = json!({
        "url": "https://github.com/rusty-ferris-club/webql",
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
    let filters = vec![
        // extract `kaplanelad` value from user -> login json pah and check if the value equal to
        // one of the given filter values
        Filter {
            query: r#""user"."login""#.to_string(),
            operation: Operation::Equal,
            values: vec!["kaplanelad".to_string()],
        },
        // extract `https://github.com/rusty-ferris-club/webql` value from url json key and check if the value equal to
        // one of the given filter values
        Filter {
            query: r#""url""#.to_string(),
            operation: Operation::Equal,
            values: vec!["https://github.com/rusty-ferris-club/webql".to_string()],
        },
        // extract `[label-1, label-2]` values from labels array and get all name values. check if
        // one of the values filter is equal to one of the name values one of the given
        // filter values
        Filter {
            query: r#""labels"|={"name"}."name""#.to_string(),
            operation: Operation::Equal,
            values: vec!["label-1".to_string()],
        },
        Filter {
            query: r#""body""#.to_string(),
            operation: Operation::Contains,
            values: vec!["example".to_string()],
        },
    ];

    println!("{:?}", jfilter::is_match_filters(&json, &filters));
}
