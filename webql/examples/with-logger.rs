use anyhow::Result;
use serde_yaml;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use webql::vendor::github::{data::Config, events::GitHub};

const CONFIG: &str = r#"
repositories:
  pull_request:
    - owner: "rusty-ferris-club"
      repo: "rust-starter" 
      priority: 1
      filters: # and condition
      - query: '"user"."login"'
        operation: =
        values: # or condition
        - kaplanelad
      - query: '"title"'
        operation: =
        values: # or condition
        - test
      - query: '"labels"|={"name"}."name"'
        operation: ~
        values: # or condition
        - enhancement
"#;

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let gh = GitHub::new().unwrap();
    let config: Config = serde_yaml::from_str(CONFIG)?;
    let result = gh.get_events(&config, 24 * 60);

    match result {
        Ok(events) => {
            println!("count event found: {:#?}", events);
        }
        Err(e) => println!("err: {:?}", e),
    };
    Ok(())
}
