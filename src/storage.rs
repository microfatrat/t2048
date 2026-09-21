//! Best-score persistence.
//!
//! Every operation here is best-effort: a missing or read-only home directory
//! must never stop somebody from playing.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

/// The file the best score is stored in, if a data directory can be found.
pub fn best_score_path() -> Option<PathBuf> {
    Some(data_dir()?.join("best"))
}

/// Where the platform puts per-user application data.
fn data_dir() -> Option<PathBuf> {
    resolve_data_dir(
        cfg!(windows),
        non_empty_env("XDG_DATA_HOME"),
        non_empty_env("APPDATA"),
        non_empty_env("HOME"),
    )
}

/// Picks the data directory from the environment.
///
/// Split out from [`data_dir`] so every platform's branch can be tested on any
/// host: `windows` is passed in rather than read from `cfg!` here.
fn resolve_data_dir(
    windows: bool,
    xdg_data_home: Option<OsString>,
    appdata: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    // XDG first, so a user can override the location anywhere.
    if let Some(dir) = xdg_data_home {
        return Some(PathBuf::from(dir).join("t2048"));
    }
    if windows {
        // %APPDATA%\t2048, e.g. C:\Users\me\AppData\Roaming\t2048
        if let Some(dir) = appdata {
            return Some(PathBuf::from(dir).join("t2048"));
        }
        // Git Bash and friends set HOME but not always APPDATA.
        if let Some(home) = home {
            return Some(
                PathBuf::from(home)
                    .join("AppData")
                    .join("Roaming")
                    .join("t2048"),
            );
        }
        return None;
    }
    // ~/.local/share/t2048, the XDG default.
    home.map(|home| PathBuf::from(home).join(".local/share/t2048"))
}

fn non_empty_env(key: &str) -> Option<OsString> {
    std::env::var_os(key).filter(|value| !value.is_empty())
}

/// Reads the stored best score, or `0` when there is none.
pub fn load_best() -> u64 {
    best_score_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

/// Stores the best score, ignoring any failure to write it.
pub fn save_best(score: u64) {
    let Some(path) = best_score_path() else {
        return;
    };
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    write_best(&path, score);
}

/// Writes the score through a temporary file, so a crash cannot leave the real
/// file truncated or half-written.
fn write_best(path: &Path, score: u64) {
    let text = score.to_string();
    let temp = path.with_extension("tmp");
    if fs::write(&temp, &text).is_err() {
        return;
    }
    if fs::rename(&temp, path).is_ok() {
        return;
    }
    // Windows refuses to rename over an existing file, so drop the old file
    // and try once more. Either way this stays best-effort.
    let _ = fs::remove_file(path);
    if fs::rename(&temp, path).is_err() {
        let _ = fs::remove_file(&temp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn os(text: &str) -> Option<OsString> {
        Some(OsString::from(text))
    }

    /// The app directory name, whichever platform chose it.
    fn app_dir(path: &Path) -> String {
        path.file_name().unwrap().to_string_lossy().into_owned()
    }

    /// A path with forward slashes, so assertions hold on any host. The
    /// Windows branches are exercised on Linux too, where `is_absolute` and
    /// friends would follow Unix rules and give the wrong answer.
    fn slashes(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    #[test]
    fn unix_uses_the_xdg_default() {
        let dir = resolve_data_dir(false, None, None, os("/home/me")).unwrap();
        assert_eq!(slashes(&dir), "/home/me/.local/share/t2048");
        assert_eq!(app_dir(&dir), "t2048");
    }

    #[test]
    fn xdg_data_home_wins_everywhere() {
        for windows in [false, true] {
            let dir = resolve_data_dir(
                windows,
                os("/custom/data"),
                os(r"C:\Users\me\AppData\Roaming"),
                os("/home/me"),
            )
            .unwrap();
            assert_eq!(slashes(&dir), "/custom/data/t2048");
        }
    }

    #[test]
    fn windows_uses_appdata() {
        let dir = resolve_data_dir(
            true,
            None,
            os(r"C:\Users\me\AppData\Roaming"),
            os(r"C:\Users\me"),
        )
        .unwrap();
        assert_eq!(slashes(&dir), "C:/Users/me/AppData/Roaming/t2048");
        assert_eq!(app_dir(&dir), "t2048");
    }

    #[test]
    fn windows_falls_back_to_home_without_appdata() {
        // Git Bash sets HOME but not always APPDATA; the result must still be
        // a full path inside the user's profile, not a relative one.
        let dir = resolve_data_dir(true, None, None, os(r"C:\Users\me")).unwrap();
        assert_eq!(slashes(&dir), "C:/Users/me/AppData/Roaming/t2048");
        assert_eq!(app_dir(&dir), "t2048");
    }

    #[test]
    fn no_environment_means_no_path_instead_of_a_relative_one() {
        assert_eq!(resolve_data_dir(false, None, None, None), None);
        assert_eq!(resolve_data_dir(true, None, None, None), None);
    }

    #[test]
    fn an_empty_variable_is_ignored() {
        // `XDG_DATA_HOME=` must not be treated as a valid directory. The
        // `non_empty_env` filter is what enforces that for the real process.
        assert_eq!(non_empty_env("XGAME_DEFINITELY_NOT_SET"), None);
        let dir = resolve_data_dir(false, None, None, os("/home/me")).unwrap();
        assert_eq!(slashes(&dir), "/home/me/.local/share/t2048");
    }

    #[test]
    fn the_best_score_lives_in_a_data_directory() {
        if let Some(path) = best_score_path() {
            assert_eq!(path.file_name().unwrap(), "best");
            assert_eq!(app_dir(path.parent().unwrap()), "t2048");
        }
    }

    #[test]
    fn loading_never_panics() {
        // Whatever the environment looks like, this must return a number.
        let _ = load_best();
    }

    #[test]
    fn a_best_score_round_trips_through_a_file() {
        let dir = std::env::temp_dir().join(format!("t2048-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("best");

        write_best(&path, 1234);
        assert_eq!(fs::read_to_string(&path).unwrap(), "1234");

        // Writing again replaces the old value and leaves no temp file.
        write_best(&path, 5678);
        assert_eq!(fs::read_to_string(&path).unwrap(), "5678");
        assert!(!path.with_extension("tmp").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
