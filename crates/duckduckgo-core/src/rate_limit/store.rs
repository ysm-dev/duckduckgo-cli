use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use url::Url;

use super::RateLimitState;
use crate::{Error, Result};

/// Owns the layout of state and lock files for one egress identity (direct
/// or proxy). Stateless across invocations; pure pathing utility plus
/// `read_state`/`write_state` helpers that touch the filesystem under the
/// caller's responsibility for synchronisation.
#[derive(Clone, Debug)]
pub struct StateStore {
    state_path: PathBuf,
    lock_path: PathBuf,
}

impl StateStore {
    pub fn new(state_dir: PathBuf, proxy: Option<&str>) -> Self {
        let file = proxy.map_or_else(
            || "rate-limit.json".to_owned(),
            |p| format!("rate-limit-{}.json", proxy_hash(p)),
        );
        Self {
            state_path: state_dir.join(file),
            lock_path: state_dir.join("rate-limit.lock"),
        }
    }

    /// Open and `O_CREAT|O_RDWR` the lock file with mode 0600 (POSIX).
    /// The caller is expected to wrap the returned `File` in
    /// `fd_lock::RwLock` and acquire the appropriate guard.
    pub fn open_lock_file(&self) -> Result<File> {
        if let Some(parent) = self.lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        open_private(&self.lock_path)
    }

    pub fn read_state(&self) -> RateLimitState {
        let mut state: RateLimitState = std::fs::read_to_string(&self.state_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        state.sanitize(OffsetDateTime::now_utc());
        state
    }

    pub fn write_state(&self, state: &RateLimitState) -> Result<()> {
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.state_path.with_extension("json.tmp");
        let json = serde_json::to_vec(state).map_err(|e| Error::Io(e.to_string()))?;
        open_private(&tmp)?.write_all(&json)?;
        std::fs::rename(tmp, &self.state_path)?;
        Ok(())
    }
}

fn open_private(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}

fn proxy_hash(proxy: &str) -> String {
    let normalized = normalize_proxy(proxy).unwrap_or_else(|| proxy.to_owned());
    Sha256::digest(normalized.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn normalize_proxy(proxy: &str) -> Option<String> {
    let url = Url::parse(proxy).ok()?;
    let scheme = url.scheme().to_ascii_lowercase();
    let host = url.host_str()?.to_ascii_lowercase();
    let default_port = matches!(
        (scheme.as_str(), url.port()),
        ("http", Some(80)) | ("https", Some(443)) | ("socks5" | "socks5h", Some(1080)) | (_, None)
    );
    let port = if default_port {
        String::new()
    } else {
        format!(":{}", url.port()?)
    };
    let path = if url.path() == "/" {
        ""
    } else {
        url.path().trim_end_matches('/')
    };
    let query = url.query().map_or_else(String::new, |q| format!("?{q}"));
    Some(format!("{scheme}://{host}{port}{path}{query}"))
}
