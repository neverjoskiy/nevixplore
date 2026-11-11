//! Settings load/save, and the paths the app needs.
//!
//! Only the list in PROMPT.md §4.3 is persisted. Window geometry, tabs,
//! history, sort state and the last location are deliberately left out: the
//! app always opens at the user's home directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Resolved, read-only locations for this machine.
#[derive(Debug, Clone)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub home: PathBuf,
}

impl Paths {
    /// Uses `%APPDATA%\NevIxplore`, falling back to the home directory.
    pub fn resolve() -> Self {
        let home = home_dir();
        let cfg_root = std::env::var_os("APPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_CONFIG_HOME")
                    .filter(|value| !value.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| home.join(".config"));
        Self {
            config_dir: cfg_root.join("NevIxplore"),
            home,
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.toml")
    }

    /// The directory the app opens at, falling back to the drive root when the
    /// profile folder is missing or unreadable.
    pub fn start_dir(&self) -> PathBuf {
        if self.home.is_dir() {
            self.home.clone()
        } else {
            PathBuf::from(r"C:\")
        }
    }
}

/// Everything the app remembers between launches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Prefs {
    pub theme: Theme,
    pub accent: Option<String>,
    pub density: Density,
    pub columns: Vec<Column>,
    pub sidebar_width: u16,
    pub show_sidebar: bool,
    pub show_preview: bool,
    pub preview_enabled: bool,
    pub index_roots: Vec<String>,
    pub trash_policy: TrashPolicy,
    pub max_content_search_bytes: u64,
    pub terminal: Terminal,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            accent: None,
            density: Density::Comfortable,
            columns: Column::defaults(),
            sidebar_width: 220,
            show_sidebar: true,
            show_preview: false,
            preview_enabled: true,
            index_roots: Vec::new(),
            trash_policy: TrashPolicy::RecycleBin,
            max_content_search_bytes: 16 * 1024 * 1024,
            terminal: Terminal::WindowsTerminal,
        }
    }
}

impl Prefs {
    /// Reads settings, falling back to defaults on any problem.
    ///
    /// A corrupt file is reported once and left on disk, so the user can fix
    /// it by hand instead of losing their configuration silently.
    pub fn load(paths: &Paths) -> Self {
        let file = paths.settings_file();
        let text = match std::fs::read_to_string(&file) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(err) => {
                tracing::warn!(%err, path = %file.display(), "settings unreadable, using defaults");
                return Self::default();
            }
        };
        let mut prefs: Self = match toml::from_str(&text) {
            Ok(prefs) => prefs,
            Err(err) => {
                tracing::warn!(%err, path = %file.display(), "settings invalid, using defaults");
                Self::default()
            }
        };
        prefs.normalize();
        prefs
    }

    /// Writes settings atomically: temp file, then rename.
    pub fn save(&self, paths: &Paths) -> std::io::Result<()> {
        std::fs::create_dir_all(&paths.config_dir)?;
        let body = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let target = paths.settings_file();
        let temp = target.with_extension("toml.tmp");
        std::fs::write(&temp, body)?;
        std::fs::rename(&temp, &target)
    }

    /// Clamps values that the clamp-on-write rule keeps in range.
    pub fn normalize(&mut self) {
        self.sidebar_width = self.sidebar_width.clamp(160, 520);
        self.max_content_search_bytes = self
            .max_content_search_bytes
            .clamp(64 * 1024, 512 * 1024 * 1024);
        if self.columns.is_empty() {
            self.columns = Column::defaults();
        }
        self.index_roots.retain(|root| !root.trim().is_empty());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Density {
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrashPolicy {
    RecycleBin,
    Permanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Terminal {
    WindowsTerminal,
    PowerShell,
    Cmd,
}

/// A column in the file list, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Column {
    Name,
    Size,
    Modified,
    Created,
    Kind,
    Attributes,
}

impl Column {
    pub fn defaults() -> Vec<Self> {
        vec![Self::Name, Self::Size, Self::Modified, Self::Kind]
    }
}

/// The home directory, independent of the config root so tests can reason
/// about it. `USERPROFILE` is set on every Windows logon.
fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE")?;
            let path = std::env::var_os("HOMEPATH")?;
            let mut joined = PathBuf::from(drive);
            joined.push(path);
            Some(joined)
        })
        .unwrap_or_else(|| Path::new(r"C:\").to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(name: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("nv-prefs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths {
            config_dir: dir,
            home: std::env::temp_dir(),
        }
    }

    #[test]
    fn round_trips_through_disk() {
        let paths = temp_paths("roundtrip");
        let mut prefs = Prefs::default();
        prefs.theme = Theme::Dark;
        prefs.show_preview = true;
        prefs.index_roots = vec![r"C:\work".into()];
        prefs.save(&paths).expect("save");

        let loaded = Prefs::load(&paths);
        assert_eq!(loaded, prefs);
    }

    #[test]
    fn missing_file_yields_defaults() {
        let paths = temp_paths("missing");
        assert_eq!(Prefs::load(&paths), Prefs::default());
    }

    #[test]
    fn corrupt_file_yields_defaults_and_keeps_the_original() {
        let paths = temp_paths("corrupt");
        std::fs::create_dir_all(&paths.config_dir).expect("mkdir");
        std::fs::write(paths.settings_file(), "theme = [not a theme").expect("write");

        assert_eq!(Prefs::load(&paths), Prefs::default());
        assert!(paths.settings_file().exists(), "corrupt file must survive");
    }

    #[test]
    fn unknown_keys_do_not_break_loading() {
        let paths = temp_paths("forward-compat");
        std::fs::create_dir_all(&paths.config_dir).expect("mkdir");
        std::fs::write(
            paths.settings_file(),
            "theme = \"dark\"\nfuture-flag = true\n",
        )
        .expect("write");

        assert_eq!(Prefs::load(&paths).theme, Theme::Dark);
    }

    #[test]
    fn normalize_clamps_and_repairs() {
        let mut prefs = Prefs {
            sidebar_width: 5,
            max_content_search_bytes: 1,
            columns: Vec::new(),
            index_roots: vec!["  ".into(), r"D:\ok".into()],
            ..Prefs::default()
        };
        prefs.normalize();

        assert_eq!(prefs.sidebar_width, 160);
        assert_eq!(prefs.max_content_search_bytes, 64 * 1024);
        assert_eq!(prefs.columns, Column::defaults());
        assert_eq!(prefs.index_roots, vec![r"D:\ok".to_string()]);
    }

    #[test]
    fn settings_file_lives_in_the_app_folder() {
        let paths = Paths {
            config_dir: PathBuf::from(r"C:\Users\x\AppData\Roaming\NevIxplore"),
            home: PathBuf::from(r"C:\Users\x"),
        };
        assert_eq!(
            paths.settings_file(),
            PathBuf::from(r"C:\Users\x\AppData\Roaming\NevIxplore\settings.toml")
        );
    }
}
