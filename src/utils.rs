use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::OperatingSystem;

pub trait ExpandTilde {
    fn expand_tilde_path(&self) -> Result<PathBuf, String>;
}

impl<P: AsRef<Path>> ExpandTilde for P {
    fn expand_tilde_path(&self) -> Result<PathBuf, String> {
        let path_str = self.as_ref().to_string_lossy().to_string();
        if path_str.starts_with("~") {
            if let Some(home_dir) = dirs::home_dir() {
                let relative_path = path_str.strip_prefix("~").unwrap_or(&path_str);
                Ok(home_dir.join(relative_path.trim_start_matches('/')))
            } else {
                Err("Home directory not found".to_string())
            }
        } else {
            Ok(self.as_ref().to_path_buf())
        }
    }
}

pub trait MakeAbsolute {
    fn make_absolute(&self) -> Result<PathBuf, String>;
}

impl<P: AsRef<Path>> MakeAbsolute for P {
    fn make_absolute(&self) -> Result<PathBuf, String> {
        let path = self.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())
                .map(|current_dir| current_dir.join(path))
        }
    }
}

pub fn get_current_os() -> OperatingSystem {
    if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOS
    } else if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else {
        // Fallback for unknown OS, or panic if not supported
        panic!(
            "{} Unsupported operating system for Dotman.",
            "Error:".red().bold()
        );
    }
}
