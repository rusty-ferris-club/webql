//! GitHub client
use anyhow::Result;
use chrono::{DateTime, Utc};
#[cfg(test)]
use mockall::{automock, predicate::*};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
    redirect::Policy,
};
use serde_json::Value;
use tracing::debug;

use super::utils;

const GITHUB_USER_AGENT: &str = "webql-rs";

#[cfg_attr(test, automock)]
pub trait GithubClientInterface {
    fn get_all_prs(&self, owner: &str, repo_name: &str, since: DateTime<Utc>)
        -> Result<Vec<Value>>;
    fn get_issue_comments(
        &self,
        issue_id: i64,
        owner: &str,
        repo_name: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Value>>;
    fn get_issue_events(
        &self,
        issue_id: i64,
        owner: &str,
        repo_name: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Value>>;
}

pub struct GitHubClient {
    host: String,
    client: Client,
}

/// List of GitHub usage endpoints
enum Endpoint {
    ListPr(String, String, i64),
    IssueComments(String, String, i64, i64, DateTime<Utc>),
    IssueEvents(String, String, i64, i64),
}

impl Endpoint {
    //// Concat parameters and query string for GitHub request
    fn get_url(self) -> String {
        match self {
            Self::ListPr(owner, repo, page) => {
                let query_args = vec![("page", page)];
                let query = serde_urlencoded::to_string(&query_args).unwrap();
                format!("repos/{}/{}/pulls?{}", owner, repo, query)
            }
            Self::IssueComments(owner, repo, issue_id, page, since) => {
                let query_args = vec![("since", since.to_rfc3339()), ("page", page.to_string())];
                let query = serde_urlencoded::to_string(&query_args).unwrap();
                format!(
                    "repos/{}/{}/issues/{}/comments?{}",
                    owner, repo, issue_id, query
                )
            }
            Self::IssueEvents(owner, repo, issue_id, page) => {
                let query_args = vec![("page", page.to_string())];
                let query = serde_urlencoded::to_string(&query_args).unwrap();
                format!(
                    "repos/{}/{}/issues/{}/events?{}",
                    owner, repo, issue_id, query
                )
            }
        }
    }
}

impl GitHubClient {
    /// Create new GitHub client
    ///
    /// # Arguments
    /// * `host` - GitHub Host
    /// * `token` - GitHub token
    ///
    /// # Errors
    /// - when could not create new client instance
    pub fn new(host: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let client = Client::builder()
            .user_agent(GITHUB_USER_AGENT)
            .redirect(Policy::none())
            .default_headers(headers)
            .build()?;

        Ok(Self {
            host: host.to_string(),
            client,
        })
    }
}
impl GithubClientInterface for GitHubClient {
    /// Get GitHub pull request with pagination.
    ///
    /// # Arguments
    /// * `owner` - Repository owner name
    /// * `repo_name` - Repository name
    /// * `since` - Only get pull request after the given time [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - when could not get pull request from github
    fn get_all_prs(
        &self,
        owner: &str,
        repo_name: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Value>> {
        let prs = {
            let mut page = 1;
            let mut prs: Vec<Value> = vec![];
            loop {
                let endpoint = format!(
                    "{}/{}",
                    self.host,
                    Endpoint::ListPr(owner.to_string(), repo_name.to_string(), page).get_url()
                );
                debug!(message = "create http request", endpoint, page);
                let response = self.client.get(&endpoint).send()?;

                debug!(
                    message = "response status code",
                    endpoint,
                    status = format!("{}", response.status())
                );

                if response.status().is_success() {
                    let prs_response: Vec<Value> = response.json()?;
                    debug!(
                        message = "response status code",
                        endpoint,
                        page,
                        pr_count = prs_response.len(),
                    );
                    if prs_response.is_empty() {
                        debug!(message = "pull request not found", endpoint, page);
                        break;
                    }

                    prs.extend(
                        prs_response
                            .iter()
                            .filter(|pr| {
                                pr.get("updated_at").map_or(false, |d| {
                                    match utils::parse_to_date_time(d) {
                                        Ok(dt) => dt > since,
                                        Err(e) => {
                                            debug!(
                                                message = "could not convert filed to date time",
                                                endpoint,
                                                page,
                                                err = e.to_string(),
                                            );
                                            false
                                        }
                                    }
                                })
                            })
                            .map(std::clone::Clone::clone)
                            .collect::<Vec<_>>(),
                    );
                } else {
                    break;
                }
                page += 1;
            }
            prs
        };

        debug!(
            message = format!("total pr {}", prs.len()),
            owner = &owner,
            repo = &repo_name
        );

        Ok(prs)
    }

    /// Get GitHub issue comments with pagination.
    ///
    /// # Arguments
    /// * `issue_id` - Issue ID
    /// * `owner` - Repository owner name
    /// * `repo_name` - Repository name
    /// * `since` - Only get issue comments after the given time
    ///   [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - when could not get issue comments from github
    fn get_issue_comments(
        &self,
        issue_id: i64,
        owner: &str,
        repo_name: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Value>> {
        let comments = {
            let mut page = 1;
            let mut comments: Vec<Value> = vec![];
            loop {
                let endpoint = format!(
                    "{}/{}",
                    self.host,
                    Endpoint::IssueComments(
                        owner.to_string(),
                        repo_name.to_string(),
                        issue_id,
                        page,
                        since
                    )
                    .get_url()
                );
                debug!(message = "create http request", endpoint, page, issue_id);
                let response = self.client.get(&endpoint).send()?;

                debug!(
                    message = "response status code",
                    endpoint,
                    issue_id,
                    status = format!("{}", response.status())
                );

                if response.status().is_success() {
                    let comments_response: Vec<Value> = response.json()?;
                    debug!(
                        message = "response status code",
                        endpoint,
                        page,
                        issue_id,
                        comments_count = comments_response.len(),
                    );
                    if comments_response.is_empty() {
                        debug!(message = "comments not found", endpoint, page, issue_id);
                        break;
                    }
                    comments.extend(comments_response);
                } else {
                    break;
                }
                page += 1;
            }
            comments
        };

        Ok(comments)
    }

    /// Get GitHub issue events with pagination.
    ///
    /// # Arguments
    /// * `issue_id` - Issue ID
    /// * `owner` - Repository owner name
    /// * `repo_name` - Repository name
    /// * `since` - Only get issue events after the given time [`DateTime<Utc>`]
    ///
    /// # Errors
    /// - when could not get issue events from github
    fn get_issue_events(
        &self,
        issue_id: i64,
        owner: &str,
        repo_name: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Value>> {
        let events = {
            let mut page = 1;
            let mut events: Vec<Value> = vec![];
            loop {
                let endpoint = format!(
                    "{}/{}",
                    self.host,
                    Endpoint::IssueEvents(
                        owner.to_string(),
                        repo_name.to_string(),
                        issue_id,
                        page,
                    )
                    .get_url()
                );
                debug!(message = "create http request", endpoint, page);
                let response = self.client.get(&endpoint).send()?;

                debug!(
                    message = "response status code",
                    endpoint,
                    issue_id,
                    status = format!("{}", response.status())
                );

                if response.status().is_success() {
                    let events_response: Vec<Value> = response.json()?;
                    debug!(
                        message = "response status code",
                        endpoint,
                        page,
                        issue_id,
                        events_count = events_response.len(),
                    );
                    if events_response.is_empty() {
                        debug!(message = "events not found", endpoint, page, issue_id);
                        break;
                    }
                    events.extend(
                        events_response
                            .iter()
                            .filter(|pr| {
                                pr.get("created_at").map_or(false, |d| {
                                    match utils::parse_to_date_time(d) {
                                        Ok(dt) => dt > since,
                                        Err(e) => {
                                            debug!(
                                                message = "could not convert filed to date time",
                                                endpoint,
                                                page,
                                                issue_id,
                                                err = e.to_string(),
                                            );
                                            false
                                        }
                                    }
                                })
                            })
                            .map(std::clone::Clone::clone)
                            .collect::<Vec<_>>(),
                    );
                } else {
                    break;
                }
                page += 1;
            }
            events
        };

        Ok(events)
    }
}

#[cfg(test)]
mod test_client {

    use chrono::{naive::NaiveDate, DateTime, Duration, Utc};
    use httpmock::prelude::*;
    use insta::{assert_debug_snapshot, with_settings};
    use serde_json::{json, Value};

    use super::{GitHubClient, GithubClientInterface};

    #[test]
    fn can_get_all_prs() {
        let server = MockServer::start();

        let now = Utc::now();
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/pulls")
                .query_param("page", "1");
            then.status(200).json_body(vec![
                json!({
                    "id": 1,
                    "updated_at": now + Duration::minutes(1),
                }),
                json!({
                    "id": 2,
                    "updated_at": now + Duration::minutes(2),
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/pulls")
                .query_param("page", "2");
            then.status(200).json_body(vec![
                json!({
                    "id": 3,
                    "updated_at": now + Duration::minutes(1),
                }),
                json!({
                    "id": 4,
                    "updated_at": now - Duration::minutes(2),
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/pulls")
                .query_param("page", "3");
            then.status(200).json_body(Value::Array(vec![]));
        });

        let gh: Box<dyn GithubClientInterface> =
            Box::new(GitHubClient::new(&server.base_url(), "1234").unwrap());

        with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}[A-Z][0-9]{1,2}:[0-9]{1,2}:[0-9]{1,2}.[0-9]*Z", "DATE")
        ]}, {
        assert_debug_snapshot!(gh.get_all_prs("rusty-ferris-club", "webql", now));
        });
    }

    #[test]
    fn can_get_issue_comments() {
        let server = MockServer::start();

        let naivedatetime_utc = NaiveDate::from_ymd(2000, 1, 12).and_hms(2, 0, 0);
        let time = DateTime::<Utc>::from_utc(naivedatetime_utc, Utc);

        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/comments")
                .query_param("page", "1")
                .query_param("since", "2000-01-12T02:00:00+00:00");
            then.status(200).json_body(vec![
                json!({
                    "id": 1,
                }),
                json!({
                    "id": 2,
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/comments")
                .query_param("page", "2")
                .query_param("since", "2000-01-12T02:00:00+00:00");

            then.status(200).json_body(vec![
                json!({
                    "id": 3,
                }),
                json!({
                    "id": 4,
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/comments")
                .query_param("page", "3")
                .query_param("since", "2000-01-12T02:00:00+00:00");

            then.status(200).json_body(Value::Array(vec![]));
        });

        let gh: Box<dyn GithubClientInterface> =
            Box::new(GitHubClient::new(&server.base_url(), "1234").unwrap());

        assert_debug_snapshot!(gh.get_issue_comments(1, "rusty-ferris-club", "webql", time));
    }

    #[test]
    fn can_get_issue_events() {
        let server = MockServer::start();

        let now = Utc::now();
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/events")
                .query_param("page", "1");
            then.status(200).json_body(vec![
                json!({
                    "id": 1,
                    "created_at": now + Duration::minutes(1),
                }),
                json!({
                    "id": 2,
                    "created_at": now + Duration::minutes(2),
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/events")
                .query_param("page", "2");
            then.status(200).json_body(vec![
                json!({
                    "id": 3,
                    "created_at": now + Duration::minutes(1),
                }),
                json!({
                    "id": 4,
                    "created_at": now - Duration::minutes(2),
                }),
            ]);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/repos/rusty-ferris-club/webql/issues/1/events")
                .query_param("page", "3");
            then.status(200).json_body(Value::Array(vec![]));
        });

        let gh: Box<dyn GithubClientInterface> =
            Box::new(GitHubClient::new(&server.base_url(), "1234").unwrap());

        with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}[A-Z][0-9]{1,2}:[0-9]{1,2}:[0-9]{1,2}.[0-9]*Z", "DATE")
        ]}, {
        assert_debug_snapshot!(gh.get_issue_events(1, "rusty-ferris-club", "webql", now));
        });
    }
}
