use std::env;
use std::path::PathBuf;

use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct RuntimePaths {
    pub config_file: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
}

pub fn runtime_paths(config_override: Option<PathBuf>) -> Result<RuntimePaths> {
    Ok(RuntimePaths {
        config_file: config_override.unwrap_or_else(default_config_file),
        state_dir: default_state_dir()?,
        cache_dir: default_cache_dir()?,
    })
}

fn default_config_file() -> PathBuf {
    #[cfg(windows)]
    {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(home_dir_lossy)
            .join("duckduckgo-cli")
            .join("config.jsonc")
    }
    #[cfg(not(windows))]
    {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir_lossy().join(".config"))
            .join("duckduckgo-cli")
            .join("config.jsonc")
    }
}

fn default_state_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    let dir = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(home_dir_lossy)
        .join("duckduckgo-cli")
        .join("state");
    #[cfg(not(windows))]
    let dir = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir_lossy().join(".local").join("state"))
        .join("duckduckgo-cli");
    Ok(dir)
}

fn default_cache_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    let dir = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(home_dir_lossy)
        .join("duckduckgo-cli")
        .join("cache");
    #[cfg(not(windows))]
    let dir = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir_lossy().join(".cache"))
        .join("duckduckgo-cli");
    Ok(dir)
}

fn home_dir_lossy() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn ensure_parent(path: &std::path::Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::Io(format!("{} has no parent directory", path.display())))?;
    std::fs::create_dir_all(parent)?;
    Ok(())
}
