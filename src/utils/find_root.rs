use crate::metainfo::info_reader::{InfoError, read_validate_info};
use crate::utils::globals;
use crate::utils::log;
use std::path::{Path, PathBuf};

/// Find the root directory of a sekai by finding "location": "home"
/// in nearest `.dir_info/info.json`
pub fn check_home(sekai_path: &Path) -> Result<Option<PathBuf>, InfoError> {
    let mut current = sekai_path.to_path_buf();
    // Check for info.json in current directory
    let info_path = current.join(".dir_info/info.json");
    match read_validate_info(&info_path) {
        Ok(info) => {
            if info.location == "HOME" {
                Ok(Some(current))
            } else {
                log::log_warning(
                    "SEKAI",
                    &format!(
                        "Found info.json at {}, but location is not 'HOME': {}",
                        info_path.display(),
                        info.location
                    ),
                );
                Ok(None)
            }
        }
        Err(InfoError::NotFound(_)) => {
            log::log_warning(
                "SEKAI",
                &format!(
                    "No info.json found at {}, checking parent directory",
                    info_path.display()
                ),
            );
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Returns the home directory
pub fn get_home(sekai_path: &Path) -> Option<PathBuf> {
    match check_home(sekai_path) {
        Ok(Some(home)) => Some(home),
        Ok(None) => None,
        Err(e) => {
            log::log_error("SEKAI", &format!("Error finding Sekai home: {e}"));
            None
        }
    }
}

/// Converts an absolute path to a path relative to WORLD_DIR
/// Returns the original path if WORLD_DIR isn't set or if the path isn't within WORLD_DIR
pub fn relative_deemak_path(path: &Path) -> PathBuf {
    let world_dir = globals::get_sekai_dir();

    path.strip_prefix(&world_dir)
        .map(|relative_path| {
            if relative_path.components().count() == 0 {
                PathBuf::from("HOME")
            } else {
                PathBuf::from("HOME").join(relative_path)
            }
        })
        .unwrap_or_else(|_| path.to_path_buf())
}
