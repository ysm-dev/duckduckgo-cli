use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fd_lock::RwLock;
use sha2::{Digest, Sha256};
use url::Url;

use super::RateLimitState;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct StateStore {
    state_path: PathBuf,
    lock_path: PathBuf,
}

pub struct LockedStore {
    state_path: PathBuf,
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

    pub fn with_locked<T>(&self, f: impl FnOnce(&mut LockedStore) -> Result<T>) -> Result<T> {
        if let Some(parent) = self.lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = open_private(&self.lock_path)?;
        let mut lock = RwLock::new(file);
        let _guard = lock.write().map_err(|e| Error::Io(e.to_string()))?;
        let mut store = LockedStore {
            state_path: self.state_path.clone(),
        };
        f(&mut store)
    }
}

impl LockedStore {
    pub fn read_state(&mut self) -> RateLimitState {
        std::fs::read_to_string(&self.state_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn write_state(&mut self, state: &RateLimitState) -> Result<()> {
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
