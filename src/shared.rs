use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Deserialize)]
pub struct Config {
    pub folder: String,
    pub min_minutes: f64,
    pub max_minutes: f64,
}
#[derive(Serialize, Deserialize)]
pub struct State {
    pub status: String,
    pub deadline: Option<f64>,
    pub pid: u32,
}

pub fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}
pub fn config_path() -> PathBuf {
    env::var_os("OMARCHY_WALLPAPERS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home().join(".config"))
                .join("omarchy-wallpapers/settings.json")
        })
}
pub fn config() -> Result<Config, String> {
    let p = config_path();
    let c: Config =
        serde_json::from_slice(&fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?)
            .map_err(|e| e.to_string())?;
    if !c.min_minutes.is_finite()
        || !c.max_minutes.is_finite()
        || c.min_minutes <= 0.0
        || c.max_minutes < c.min_minutes
    {
        return Err("Invalid wallpaper interval".into());
    }
    Ok(c)
}
pub fn folder(c: &Config) -> PathBuf {
    if c.folder == "~" {
        home()
    } else if let Some(rest) = c.folder.strip_prefix("~/") {
        home().join(rest)
    } else {
        PathBuf::from(&c.folder)
    }
}
pub fn state_path() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
        .join("omarchy-wallpapers-rust-state.json")
}
pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
pub fn images(folder: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|v| v.to_str())
                    .map(|s| {
                        matches!(
                            s.to_ascii_lowercase().as_str(),
                            "jpg" | "jpeg" | "png" | "webp" | "bmp"
                        )
                    })
                    .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}
pub fn current() -> Option<PathBuf> {
    fs::canonicalize(home().join(".local/state/omarchy/current/background")).ok()
}
pub fn apply(p: &Path) -> Result<(), String> {
    let status = Command::new("omarchy")
        .args(["theme", "bg", "set"])
        .arg(p)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Не удалось установить обои: {status}"))
    }
}
pub fn shuffle() -> Result<(), String> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "kill",
            "--kill-whom=main",
            "--signal=USR1",
            "random-wallpaper.service",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
pub fn write_state(status: &str, deadline: Option<f64>) -> Result<(), String> {
    let path = state_path();
    let temp = path.with_extension("tmp");
    let data = serde_json::to_vec(&State {
        status: status.into(),
        deadline,
        pid: std::process::id(),
    })
    .map_err(|e| e.to_string())?;
    fs::write(&temp, data)
        .and_then(|_| fs::rename(temp, path))
        .map_err(|e| e.to_string())
}
