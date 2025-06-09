use std::path::{Path, PathBuf};

use crate::OperatingSystem;

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Result<PathBuf, String> {
    let path_str = path.as_ref().to_string_lossy().to_string();
    if path_str.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            let relative_path = path_str.strip_prefix("~").unwrap_or(&path_str);
            Ok(home_dir.join(relative_path.trim_start_matches('/')))
        } else {
            Err("Home directory not found".to_string())
        }
    } else {
        Ok(path.as_ref().to_path_buf())
    }
}

pub fn get_operating_system() -> Result<OperatingSystem, ()> {
    match std::env::consts::OS {
        "linux" => Ok(OperatingSystem::Linux),
        "macos" => Ok(OperatingSystem::MacOS),
        "windows" => Ok(OperatingSystem::Windows),
        _ => Err(()),
    }
}
