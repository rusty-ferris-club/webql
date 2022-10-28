use serde_derive::Deserialize;

use crate::data::Filter;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub repositories: Repositories,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Repositories {
    pub pull_request: Option<Vec<PullRequest>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PullRequest {
    pub owner: String,
    pub repo: String,
    pub priority: usize,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PullRequestResponse {
    pub number: i64,
    pub html_url: String,
    pub title: String,
    pub body: String,
    pub user: UserResponse,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserResponse {
    pub login: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IssueCommentResponse {
    pub id: i64,
    pub html_url: String,
    pub body: String,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IssueEventResponse {
    pub id: i64,
    pub event: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
