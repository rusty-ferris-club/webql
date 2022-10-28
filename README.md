# webql
[![Crates.io](https://img.shields.io/crates/v/webql?style=flat-square)](https://crates.io/crates/webql)
[![CI](https://github.com/rusty-ferris-club/webql/actions/workflows/ci.yaml/badge.svg)](https://github.com/rusty-ferris-club/webql/actions/workflows/ci.yaml)
WebQL is a library that allows to get data from multiple resources or JSON and filter the result.

## Usage 
```toml
[dependencies]
webql = { version = "0.1.0" }
```

### Feature flags
* `github` feature flag for filter pull request data.

# Examples
```rs
use serde_json::json;
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
        Filter {
            query: r#""user"."login""#.to_string(),
            operation: Operation::Equal,
            values: vec!["kaplanelad".to_string()],
        },
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
    jfilter::is_match_filters(&json, &filters)
```

[All the examples here](./example/README.MD)

# Thanks
To all [Contributors](https://github.com/rusty-ferris-club/webql/graphs/contributors) - you make this happen, thanks!

# Copyright
Copyright (c) 2022 [@kaplanelad](https://github.com/kaplanelad). See [LICENSE](LICENSE.txt) for further details.
