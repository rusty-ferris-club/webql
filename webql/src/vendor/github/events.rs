//! Fetch GitHub data
//!
//! # Example:
//! ```
#![doc = include_str!("../../../examples/github.rs")]
//! ```
use std::env;

use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use tracing::debug;

use super::{
    client::{GitHubClient, GithubClientInterface},
    data::{Config, IssueCommentResponse, IssueEventResponse, PullRequest, PullRequestResponse},
};
use crate::{
    data::{Event, EventKind},
    jfilter,
};

/// GitHub environment token name
const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
/// Default GitHub api key
pub const DEFAULT_HOST: &str = "https://api.github.com";

pub struct GitHub {
    client: Box<dyn GithubClientInterface>,
}

impl GitHub {
    /// Create new GitHub pull events. by default using [`DEFAULT_HOST`] host
    /// value and using GITHUB_TOKEN from environment variable
    ///
    /// # Errors
    /// - GITHUB_TOKEN not found
    /// - Could not initialize HTTP client
    pub fn new() -> Result<Self> {
        Self::custom(DEFAULT_HOST, None)
    }

    /// Create custom GitHub pull events
    ///
    /// # Arguments
    /// * `host` - GitHub Host
    /// * `token` - GitHub token. In case is Null, search the token from
    ///   environment variable via GITHUB_TOKEN value
    ///
    /// # Errors
    /// - GITHUB_TOKEN not found
    /// - Could not initialize HTTP client
    pub fn custom(host: &str, token: Option<String>) -> Result<Self> {
        let real_token = match token.map_or(env::var(GITHUB_TOKEN), Ok) {
            Ok(t) => t,
            Err(_e) => {
                bail!("token not provided")
            }
        };

        debug!(message = "create new github event puller", host);
        Ok(Self {
            client: Box::new(GitHubClient::new(host, &real_token)?),
        })
    }

    /// Get GitHub events.
    ///
    /// # Arguments
    /// * `config` - event [`Config`]
    /// * `minutes_ago` - From when get the data
    ///
    /// # Errors
    /// - GitHub API return an error
    /// - When filter the data
    pub fn get_events(&self, config: &Config, minutes_ago: i64) -> Result<Vec<Event>> {
        let since = Utc::now() - Duration::minutes(minutes_ago);

        let events = {
            let mut errors = vec![];
            let events = config
                .repositories
                .pull_request
                .as_ref()
                .map_or_else(std::vec::Vec::new, |repositories| {
                    repositories
                        .iter()
                        .filter_map(|pr_query| match self.get_prs_events(pr_query, since) {
                            Ok(prs) => Some(prs),
                            Err(e) => {
                                errors.push(e);
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .iter()
                .flat_map(std::clone::Clone::clone)
                .collect::<Vec<_>>();

            events
        };

        Ok(events)
    }

    /// Get GitHub pull requests
    ///
    /// # Arguments
    /// * `pr_filters` - [`PullRequest`] data
    /// * `since` - Only get pull request after the given time [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - GitHub API return an error
    /// - When filter the data
    fn get_prs_events(&self, pr_filters: &PullRequest, since: DateTime<Utc>) -> Result<Vec<Event>> {
        let mut events: Vec<Event> = vec![];

        let prs = self
            .client
            .get_all_prs(&pr_filters.owner, &pr_filters.repo, since)?;
        for pr in prs {
            let pull_request: PullRequestResponse = serde_json::from_value(pr.clone())?;

            if !jfilter::is_match_filters(&pr, &pr_filters.filters)? {
                continue;
            }

            events.extend(self.get_comments_event(pull_request.number, pr_filters, since)?);
            events.extend(self.get_issue_events(pull_request.number, pr_filters, since)?);

            events.push(Event {
                kind: EventKind::PR,
                id: pull_request.number.to_string(),
                parent_event_id: None,
                name: pull_request.title,
                link: Some(pull_request.html_url),
                date: pull_request.updated_at,
                priority: pr_filters.priority,
                row_data: pr.clone(),
            });
        }

        Ok(events)
    }

    /// # Get comments on the given issue
    ///
    /// # Arguments
    /// * `issue_id` - Issue ID
    /// * `filters` - Query [`PullRequest`]
    /// * `since` - Only get comments after the given time [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - When could not get comments from github
    /// - Could not GitHub response to [`IssueCommentResponse`]
    fn get_comments_event(
        &self,
        issue_id: i64,
        filters: &PullRequest,
        since: DateTime<Utc>,
    ) -> Result<Vec<Event>> {
        let mut events: Vec<Event> = vec![];
        let comments =
            self.client
                .get_issue_comments(issue_id, &filters.owner, &filters.repo, since)?;

        for comment_value in comments {
            let comment: IssueCommentResponse = serde_json::from_value(comment_value.clone())?;
            events.push(Event {
                kind: EventKind::PrComment,
                id: comment.id.to_string(),
                parent_event_id: Some(issue_id.to_string()),
                name: comment.body,
                link: Some(comment.html_url),
                date: comment.updated_at,
                priority: filters.priority,
                row_data: comment_value.clone(),
            });
        }

        Ok(events)
    }

    /// # Get issue Events on the given issue
    ///
    /// # Arguments
    /// * `issue_id` - Issue ID
    /// * `filters` - Query [`PullRequest`]
    /// * `since` - Only get comments after the given time [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - When could not get events from github
    /// - Could not GitHub response to [`IssueEventResponse`]
    fn get_issue_events(
        &self,
        issue_id: i64,
        filters: &PullRequest,
        since: DateTime<Utc>,
    ) -> Result<Vec<Event>> {
        let mut events: Vec<Event> = vec![];
        let events_response =
            self.client
                .get_issue_events(issue_id, &filters.owner, &filters.repo, since)?;

        for event_value in events_response {
            let event: IssueEventResponse = serde_json::from_value(event_value.clone())?;
            events.push(Event {
                kind: EventKind::PrEvent,
                id: event.id.to_string(),
                parent_event_id: Some(issue_id.to_string()),
                name: event.event,
                link: None,
                date: event.created_at,
                priority: filters.priority,
                row_data: event_value.clone(),
            });
        }
        Ok(events)
    }
}

#[cfg(test)]
mod test_events {

    use chrono::Utc;
    use insta::assert_debug_snapshot;
    use mockall::predicate::{eq, ne};
    use serde_json::json;

    use super::{Config, GitHub};
    use crate::vendor::github::{
        client::MockGithubClientInterface,
        data::{PullRequest, Repositories},
    };

    #[test]
    fn can_get_events() {
        let mut client = Box::new(MockGithubClientInterface::new());

        client
            .expect_get_all_prs()
            .with(eq("rusty-ferris-club"), eq("webql"), ne(Utc::now()))
            .returning(|_a, _b, _c| {
                Ok(vec![json!({
                    "number": 1,
                    "html_url": "https://rusty-ferris-club/webql/pulls/1",
                    "title": "pr 1",
                    "body": "",
                    "user": {
                        "login": ""
                    }
                })])
            });

        client
            .expect_get_issue_comments()
            .with(eq(1), eq("rusty-ferris-club"), eq("webql"), ne(Utc::now()))
            .returning(|_, _, _, _| {
                Ok(vec![json!({
                    "id": 1,
                    "html_url": "https://rusty-ferris-club/webql/pulls/1",
                    "body": "",
                })])
            });

        client
            .expect_get_issue_events()
            .with(eq(1), eq("rusty-ferris-club"), eq("webql"), ne(Utc::now()))
            .returning(|_, _, _, _| {
                Ok(vec![json!({
                    "id": 1,
                    "event": "name",
                })])
            });

        let gh = GitHub { client };
        let config = Config {
            repositories: Repositories {
                pull_request: Some(vec![PullRequest {
                    owner: "rusty-ferris-club".to_string(),
                    repo: "webql".to_string(),
                    priority: 1,
                    filters: vec![],
                }]),
            },
        };
        assert_debug_snapshot!(gh.get_events(&config, 10));
    }
}
