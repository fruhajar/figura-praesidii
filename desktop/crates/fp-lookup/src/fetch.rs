//! Fetching, with a cache that doubles as the offline test harness.
//!
//! Politeness is not optional here. Querying a public register about yourself is
//! unremarkable; hammering it from a tool that ships to strangers is how the endpoint
//! gets closed for everyone.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum FetchError {
    /// No cached copy and the fetcher is offline.
    Offline(String),
    Http(u16),
    Transport(String),
    /// The per-run request budget is spent.
    Budget,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline(url) => write!(f, "offline and nothing cached for {url}"),
            Self::Http(code) => write!(f, "HTTP {code}"),
            Self::Transport(e) => write!(f, "{e}"),
            Self::Budget => write!(f, "request budget for this run is spent"),
        }
    }
}

pub trait Fetcher {
    fn get(&mut self, url: &str, min_interval_ms: u64) -> Result<String, FetchError>;
    /// Requests actually sent, for the report.
    fn requests(&self) -> u32 {
        0
    }
}

pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Readable filename plus a hash, so a cache directory can be inspected by eye and
    /// still never collide.
    pub fn path_for(&self, url: &str) -> PathBuf {
        let slug: String = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .take(80)
            .collect();
        self.dir.join(format!("{slug}-{:016x}.html", fnv1a(url)))
    }

    pub fn read(&self, url: &str) -> Option<String> {
        fs::read_to_string(self.path_for(url)).ok()
    }

    pub fn write(&self, url: &str, body: &str) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.path_for(url), body)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// FNV-1a, 64-bit. Not a security hash: it only has to keep two cache filenames apart.
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Serves only what is already cached. This is what runs in CI and on a machine with
/// no egress, and it is the default in the tests.
pub struct OfflineFetcher {
    cache: Cache,
}

impl OfflineFetcher {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            cache: Cache::new(dir),
        }
    }
}

impl Fetcher for OfflineFetcher {
    fn get(&mut self, url: &str, _min_interval_ms: u64) -> Result<String, FetchError> {
        self.cache
            .read(url)
            .ok_or_else(|| FetchError::Offline(url.to_string()))
    }
}

#[cfg(feature = "http")]
pub struct HttpFetcher {
    agent: ureq::Agent,
    cache: Option<Cache>,
    last: HashMap<String, Instant>,
    budget: u32,
    used: u32,
}

#[cfg(feature = "http")]
impl HttpFetcher {
    /// The user agent names the tool and the person's own purpose. A register operator
    /// seeing this in a log should be able to tell at once that it is one individual
    /// looking themselves up.
    pub fn new(budget: u32, cache: Option<Cache>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .user_agent(concat!(
                "figura-praesidii/",
                env!("CARGO_PKG_VERSION"),
                " (self-lookup by the data subject; one person, run locally)"
            ))
            .timeout(Duration::from_secs(20))
            .build();
        Self {
            agent,
            cache,
            last: HashMap::new(),
            budget,
            used: 0,
        }
    }

    fn host(url: &str) -> String {
        url.split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .unwrap_or(url)
            .to_string()
    }
}

#[cfg(feature = "http")]
impl Fetcher for HttpFetcher {
    fn get(&mut self, url: &str, min_interval_ms: u64) -> Result<String, FetchError> {
        if let Some(cache) = &self.cache {
            if let Some(hit) = cache.read(url) {
                return Ok(hit);
            }
        }
        if self.used >= self.budget {
            return Err(FetchError::Budget);
        }

        let host = Self::host(url);
        if let Some(last) = self.last.get(&host) {
            let wait = Duration::from_millis(min_interval_ms).saturating_sub(last.elapsed());
            if !wait.is_zero() {
                std::thread::sleep(wait);
            }
        }

        self.used += 1;
        self.last.insert(host, Instant::now());

        match self.agent.get(url).call() {
            Ok(resp) => {
                let body = resp
                    .into_string()
                    .map_err(|e| FetchError::Transport(e.to_string()))?;
                if let Some(cache) = &self.cache {
                    let _ = cache.write(url, &body);
                }
                Ok(body)
            }
            Err(ureq::Error::Status(code, _)) => Err(FetchError::Http(code)),
            Err(e) => Err(FetchError::Transport(e.to_string())),
        }
    }

    fn requests(&self) -> u32 {
        self.used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_matches_the_reference_vectors() {
        assert_eq!(fnv1a("a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a("foobar"), 0x85944_171f73967e8);
    }

    #[test]
    fn cache_paths_are_readable_and_unique() {
        let c = Cache::new("/tmp/fp-cache");
        let a = c.path_for("https://or.justice.cz/x?nazev=Jana");
        let b = c.path_for("https://or.justice.cz/x?nazev=Petr");
        assert_ne!(a, b);
        assert!(a.to_string_lossy().contains("or_justice_cz"));
    }

    #[test]
    fn offline_misses_are_reported_not_guessed() {
        let mut f = OfflineFetcher::new("/tmp/fp-cache-empty");
        let err = f.get("https://or.justice.cz/nothing", 0).unwrap_err();
        assert!(matches!(err, FetchError::Offline(_)));
    }
}
