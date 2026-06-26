//! Resolve a `?config=<branch>` preview to a ready-to-bind ruleset, and bind
//! the bundled default. The resolver caches resolutions briefly and rate-limits
//! source calls so previews don't exhaust the source host's quota (spec WIZ,
//! Preview against repository refs). The default path uses [`ResolvedRuleset`]
//! built once at boot and makes no source call.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::model::Ruleset;
use super::normalize;
use super::source::RefSource;
use crate::error::{AppError, Result};

/// The fixed path of the ruleset within the source repository. Versions are
/// branches of this one file (spec WIZ).
pub const RULESET_PATH: &str = "ruleset.ron";

/// A parsed, validated, normalized ruleset ready to store and bind: its content
/// hash, the canonical JSON stored in `config_store`, and the parsed model.
#[derive(Debug, Clone)]
pub struct ResolvedRuleset {
	pub hash: String,
	pub canonical_json: String,
	pub ruleset: Ruleset,
}

impl ResolvedRuleset {
	/// Parse RON, validate the stable-id discipline, normalize, and hash.
	pub fn from_ron(source: &str) -> Result<Self> {
		let ruleset = Ruleset::from_ron(source)?;
		ruleset.validate()?;
		let canonical_json = normalize::canonical_json(&ruleset)?;
		let hash = normalize::content_hash(&canonical_json);
		Ok(Self {
			hash,
			canonical_json,
			ruleset,
		})
	}
}

/// A branch name is acceptable when it could name a git branch and carries no
/// path-traversal tricks. The host and repository are fixed by the source, so
/// this guards only the branch segment of the request.
fn validate_branch(branch: &str) -> Result<()> {
	let ok = !branch.is_empty()
		&& branch.len() <= 255
		&& !branch.starts_with('/')
		&& !branch.ends_with('/')
		&& !branch.contains("..")
		&& branch
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-'));
	if ok {
		Ok(())
	} else {
		Err(AppError::BadRequest(format!(
			"invalid branch name: {branch}"
		)))
	}
}

#[derive(Clone)]
pub struct RulesetResolver<S: RefSource> {
	source: S,
	cache: Arc<Mutex<HashMap<String, (ResolvedRuleset, Instant)>>>,
	cache_ttl: Duration,
	bucket: Arc<Mutex<TokenBucket>>,
}

impl<S: RefSource> RulesetResolver<S> {
	/// Default limits: cache a resolution for 60s, and allow a burst of 20
	/// source resolutions refilling at 20/minute.
	pub fn new(source: S) -> Self {
		Self::with_limits(source, Duration::from_secs(60), 20.0, 20.0 / 60.0)
	}

	pub fn with_limits(source: S, cache_ttl: Duration, burst: f64, refill_per_sec: f64) -> Self {
		Self {
			source,
			cache: Arc::new(Mutex::new(HashMap::new())),
			cache_ttl,
			bucket: Arc::new(Mutex::new(TokenBucket::new(burst, refill_per_sec))),
		}
	}

	/// Resolve a branch to a ready-to-bind ruleset. Returns a cached result
	/// without a source call when fresh; otherwise spends a rate-limit token.
	pub async fn resolve(&self, branch: &str) -> Result<ResolvedRuleset> {
		validate_branch(branch)?;

		if let Some(cached) = self.cached(branch) {
			return Ok(cached);
		}

		if !self.bucket.lock().unwrap().try_take() {
			return Err(AppError::RateLimited);
		}

		let commit = self.source.resolve_ref(branch).await?;
		let content = self.source.fetch_file(&commit, RULESET_PATH).await?;
		let resolved = ResolvedRuleset::from_ron(&content)?;

		self.cache
			.lock()
			.unwrap()
			.insert(branch.to_owned(), (resolved.clone(), Instant::now()));
		Ok(resolved)
	}

	fn cached(&self, branch: &str) -> Option<ResolvedRuleset> {
		let cache = self.cache.lock().unwrap();
		let (resolved, at) = cache.get(branch)?;
		(at.elapsed() < self.cache_ttl).then(|| resolved.clone())
	}
}

/// A simple token bucket: `burst` tokens, refilling at `refill_per_sec`.
struct TokenBucket {
	capacity: f64,
	tokens: f64,
	refill_per_sec: f64,
	last: Instant,
}

impl TokenBucket {
	fn new(capacity: f64, refill_per_sec: f64) -> Self {
		Self {
			capacity,
			tokens: capacity,
			refill_per_sec,
			last: Instant::now(),
		}
	}

	fn try_take(&mut self) -> bool {
		let now = Instant::now();
		let elapsed = now.duration_since(self.last).as_secs_f64();
		self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
		self.last = now;
		if self.tokens >= 1.0 {
			self.tokens -= 1.0;
			true
		} else {
			false
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};

	use super::*;

	const MINIMAL: &str = "(questions: [], rules: [])";

	#[derive(Clone)]
	struct FakeSource {
		content: String,
		resolves: Arc<AtomicUsize>,
	}

	impl FakeSource {
		fn new(content: &str) -> Self {
			Self {
				content: content.to_owned(),
				resolves: Arc::new(AtomicUsize::new(0)),
			}
		}
		fn resolve_count(&self) -> usize {
			self.resolves.load(Ordering::SeqCst)
		}
	}

	impl RefSource for FakeSource {
		async fn resolve_ref(&self, _branch: &str) -> Result<String> {
			self.resolves.fetch_add(1, Ordering::SeqCst);
			Ok("deadbeefsha".to_owned())
		}
		async fn fetch_file(&self, _commit: &str, _path: &str) -> Result<String> {
			Ok(self.content.clone())
		}
	}

	#[tokio::test]
	async fn resolves_and_caches_per_branch() {
		let source = FakeSource::new(MINIMAL);
		let resolver = RulesetResolver::new(source.clone());

		let a1 = resolver.resolve("main").await.unwrap();
		let a2 = resolver.resolve("main").await.unwrap();
		// Same branch within TTL is served from cache: one source call only.
		assert_eq!(a1.hash, a2.hash);
		assert_eq!(source.resolve_count(), 1);

		// A different branch hits the source again.
		resolver.resolve("feature/x").await.unwrap();
		assert_eq!(source.resolve_count(), 2);
	}

	#[tokio::test]
	async fn rate_limit_rejects_excess() {
		// Burst of one, no refill: the first distinct branch resolves, the next
		// is rejected.
		let resolver = RulesetResolver::with_limits(
			FakeSource::new(MINIMAL),
			Duration::from_secs(60),
			1.0,
			0.0,
		);
		resolver.resolve("one").await.unwrap();
		assert!(matches!(
			resolver.resolve("two").await,
			Err(AppError::RateLimited)
		));
	}

	#[tokio::test]
	async fn invalid_branch_is_rejected_without_a_source_call() {
		let source = FakeSource::new(MINIMAL);
		let resolver = RulesetResolver::new(source.clone());
		assert!(matches!(
			resolver.resolve("../../etc/passwd").await,
			Err(AppError::BadRequest(_))
		));
		assert_eq!(source.resolve_count(), 0);
	}

	#[tokio::test]
	async fn invalid_ruleset_content_errors() {
		let resolver = RulesetResolver::new(FakeSource::new("not valid ron"));
		assert!(resolver.resolve("main").await.is_err());
	}
}
