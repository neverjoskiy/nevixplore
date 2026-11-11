//! Typed errors and their user-facing rendering.
//!
//! Every variant maps to a short message, a longer explanation, and an OS code
//! when one exists. The UI receives [`ErrorDto`]; it never sees a raw
//! `std::io::Error`.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Errors that can reach the UI.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("cannot read {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("access denied: {path}")]
    Permission { path: PathBuf },
    #[error("not found: {path}")]
    NotFound { path: PathBuf },
    #[error("not a directory: {path}")]
    NotADirectory { path: PathBuf },
    #[error("path too long: {path}")]
    PathTooLong { path: PathBuf },
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Wraps an IO error, choosing the most specific variant for the OS code.
    ///
    /// Callers pass the path they were working on so the message can name it.
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        let path = path.as_ref().to_path_buf();
        match source.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound { path },
            std::io::ErrorKind::PermissionDenied => Self::Permission { path },
            _ if is_path_too_long(&source) => Self::PathTooLong { path },
            _ => Self::Io { path, source },
        }
    }

    /// A short message for the status bar or a toast.
    pub fn message(&self) -> String {
        match self {
            Self::Io { path, .. } => format!("Could not read {}", display(path)),
            Self::Permission { path } => format!("Access denied: {}", display(path)),
            Self::NotFound { path } => format!("Not found: {}", display(path)),
            Self::NotADirectory { path } => format!("Not a folder: {}", display(path)),
            Self::PathTooLong { path } => format!("Path is too long: {}", display(path)),
            Self::Cancelled => "Cancelled".to_string(),
            Self::Internal(msg) => msg.clone(),
        }
    }

    /// The long form, shown in the details disclosure of an error dialog.
    pub fn detail(&self) -> String {
        match self {
            Self::Io { source, .. } => source.to_string(),
            Self::Permission { .. } => {
                "Windows denied access to this item. It may be protected by a system policy or a \
                 permission on a parent folder."
                    .to_string()
            }
            Self::NotFound { .. } => {
                "The item no longer exists. It was deleted or moved after the list was built."
                    .to_string()
            }
            Self::NotADirectory { .. } => "The item is a file, not a folder.".to_string(),
            Self::PathTooLong { .. } => {
                "The full path exceeds the Windows limit. NeviXplore reads long paths directly, but \
                 some operations still need a shorter path."
                    .to_string()
            }
            Self::Cancelled => "The operation was cancelled.".to_string(),
            Self::Internal(msg) => msg.clone(),
        }
    }

    /// The Windows error code, when the failure came from the OS.
    pub fn code(&self) -> Option<i32> {
        match self {
            Self::Io { source, .. } => source.raw_os_error(),
            Self::Permission { .. } => Some(5),
            Self::NotFound { .. } => Some(2),
            Self::PathTooLong { .. } => Some(206),
            _ => None,
        }
    }

    /// The stable identifier the frontend switches on.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Io { .. } => "io",
            Self::Permission { .. } => "permission",
            Self::NotFound { .. } => "not_found",
            Self::NotADirectory { .. } => "not_a_directory",
            Self::PathTooLong { .. } => "path_too_long",
            Self::Cancelled => "cancelled",
            Self::Internal(_) => "internal",
        }
    }

    pub fn to_dto(&self) -> ErrorDto {
        ErrorDto {
            kind: self.kind().to_string(),
            message: self.message(),
            detail: self.detail(),
            path: match self {
                Self::Io { path, .. }
                | Self::Permission { path }
                | Self::NotFound { path }
                | Self::NotADirectory { path }
                | Self::PathTooLong { path } => Some(path.display().to_string()),
                Self::Cancelled | Self::Internal(_) => None,
            },
            code: self.code(),
        }
    }
}

/// The serialized shape the UI consumes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDto {
    pub kind: String,
    pub message: String,
    pub detail: String,
    pub path: Option<String>,
    pub code: Option<i32>,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_dto().serialize(serializer)
    }
}

/// Converts a `tauri::Error`, which is what `setup` and window calls return.
impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

fn is_path_too_long(err: &std::io::Error) -> bool {
    // ERROR_FILENAME_EXCED_RANGE (206) and ERROR_BUFFER_OVERFLOW (111).
    matches!(err.raw_os_error(), Some(206) | Some(111))
}

/// Shows the last component for long paths, keeps short ones whole.
fn display(path: &Path) -> String {
    let text = path.display().to_string();
    if text.chars().count() <= 64 {
        return text;
    }
    match path.file_name() {
        Some(name) => format!("…\\{}", name.to_string_lossy()),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_a_short_message() {
        let err = AppError::NotFound {
            path: PathBuf::from(r"C:\temp\gone.txt"),
        };
        assert_eq!(err.message(), r"Not found: C:\temp\gone.txt");
        assert_eq!(err.kind(), "not_found");
        assert_eq!(err.code(), Some(2));
    }

    #[test]
    fn io_errors_pick_the_specific_variant() {
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
        let err = AppError::io(r"C:\temp\gone", missing);
        assert!(matches!(err, AppError::NotFound { .. }));

        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let err = AppError::io(r"C:\temp\locked", denied);
        assert!(matches!(err, AppError::Permission { .. }));
    }

    #[test]
    fn long_paths_are_shortened_for_display() {
        let long = format!(r"C:\{}.txt", "a".repeat(200));
        let err = AppError::NotFound { path: long.into() };
        assert!(err.message().starts_with("Not found: …\\"));
    }

    #[test]
    fn serializes_to_the_ui_contract() {
        let err = AppError::Cancelled;
        let json = serde_json::to_value(&err).expect("serializes");
        assert_eq!(json["kind"], "cancelled");
        assert!(json["path"].is_null());
        assert!(json["code"].is_null());
    }
}
