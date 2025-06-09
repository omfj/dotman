use hex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn hash_file_bytes<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(&file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_vec())
}

pub fn file_checksum<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let path = file_path.as_ref();
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path is not a file",
        ));
    }

    let hash_bytes = hash_file_bytes(path)?;
    Ok(hex::encode(hash_bytes))
}

pub fn folder_checksum<P: AsRef<Path>>(dir_path: P) -> io::Result<String> {
    let root_path = dir_path.as_ref();
    if !root_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path is not a directory",
        ));
    }

    let mut hasher = Sha256::new();
    let mut entries: Vec<DirEntry> = WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // Sort entries for consistent hashing (important for folder hashes!)
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        let relative_path = path.strip_prefix(root_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to strip prefix: {}", e),
            )
        })?;

        if path.is_file() {
            let file_hash_bytes = hash_file_bytes(path)?;
            hasher.update(relative_path.to_string_lossy().as_bytes());
            hasher.update(b":");
            hasher.update(&file_hash_bytes);
            hasher.update(b"\n");
        } else if path.is_dir() && path != root_path {
            hasher.update(relative_path.to_string_lossy().as_bytes());
            hasher.update(b"/\n");
        }
    }

    let final_hash_bytes = hasher.finalize();
    Ok(hex::encode(final_hash_bytes))
}
