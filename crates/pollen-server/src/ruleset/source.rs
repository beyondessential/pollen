//! Where a ruleset is fetched from for `?config=<branch>` preview. The trait
//! lets the resolver, cache, and rate limiter be tested without network; the
//! GitHub implementation resolves a branch through the configured repository's
//! own refs (spec WIZ, Preview against repository refs).

use std::future::Future;

use serde::Deserialize;

use crate::error::{AppError, Result};

/// Resolves a branch name to a commit and fetches a file at that commit. The
/// input is always a branch *name*, never a URL — the source decides which
/// repository to consult, so a caller cannot point it elsewhere.
pub trait RefSource: Clone + Send + Sync + 'static {
	/// Resolve a branch name to a commit identifier within the source repo.
	fn resolve_ref(&self, branch: &str) -> impl Future<Output = Result<String>> + Send;
	/// Fetch a file's contents at a resolved commit.
	fn fetch_file(&self, commit: &str, path: &str) -> impl Future<Output = Result<String>> + Send;
}

/// Resolves and fetches from a single configured GitHub repository.
#[derive(Clone)]
pub struct GitHubSource {
	repo: String,
	token: Option<String>,
	client: reqwest::Client,
}

impl GitHubSource {
	pub fn new(repo: String, token: Option<String>) -> Result<Self> {
		let client = reqwest::Client::builder()
			.user_agent("pollen")
			.build()
			.map_err(|e| AppError::custom(format!("build http client: {e}")))?;
		Ok(Self {
			repo,
			token,
			client,
		})
	}
}

#[derive(Deserialize)]
struct GitRef {
	object: GitObject,
}

#[derive(Deserialize)]
struct GitObject {
	sha: String,
}

impl RefSource for GitHubSource {
	async fn resolve_ref(&self, branch: &str) -> Result<String> {
		// The host and repository are fixed here; only the branch name varies,
		// so this can never resolve to a fork or an arbitrary location.
		let url = format!(
			"https://api.github.com/repos/{}/git/ref/heads/{}",
			self.repo, branch
		);
		let mut req = self
			.client
			.get(&url)
			.header("Accept", "application/vnd.github+json")
			.header("X-GitHub-Api-Version", "2022-11-28");
		if let Some(token) = &self.token {
			req = req.bearer_auth(token);
		}
		let resp = req
			.send()
			.await
			.map_err(|e| AppError::Upstream(format!("resolve ref: {e}")))?;
		if resp.status() == reqwest::StatusCode::NOT_FOUND {
			return Err(AppError::NotFound(format!("no such branch: {branch}")));
		}
		if !resp.status().is_success() {
			return Err(AppError::Upstream(format!(
				"resolve ref: status {}",
				resp.status()
			)));
		}
		let git_ref: GitRef = resp
			.json()
			.await
			.map_err(|e| AppError::Upstream(format!("resolve ref body: {e}")))?;
		Ok(git_ref.object.sha)
	}

	async fn fetch_file(&self, commit: &str, path: &str) -> Result<String> {
		let url = format!(
			"https://raw.githubusercontent.com/{}/{}/{}",
			self.repo, commit, path
		);
		let mut req = self.client.get(&url);
		if let Some(token) = &self.token {
			req = req.bearer_auth(token);
		}
		let resp = req
			.send()
			.await
			.map_err(|e| AppError::Upstream(format!("fetch ruleset: {e}")))?;
		if resp.status() == reqwest::StatusCode::NOT_FOUND {
			return Err(AppError::NotFound(format!("no {path} at {commit}")));
		}
		if !resp.status().is_success() {
			return Err(AppError::Upstream(format!(
				"fetch ruleset: status {}",
				resp.status()
			)));
		}
		resp.text()
			.await
			.map_err(|e| AppError::Upstream(format!("fetch ruleset body: {e}")))
	}
}
