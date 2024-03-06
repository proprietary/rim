use blake3::Hasher;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub original_path: String,
    pub file_size: u64,
    pub blake3sum: String,
    pub mtime: u64,
    pub atime: u64,
    pub unix_mode: u32,
    pub uid: u32,
    pub gid: u32,
}

pub fn read_file_meta(path: &std::path::Path) -> Result<FileMetadata, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
    }
    let metadata = path.metadata()?;
    let mtime: u64 = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let atime: u64 = metadata
        .accessed()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let blake3sum = blake3sum(path)?;
    Ok(FileMetadata {
        original_path: path.to_string_lossy().to_string(),
        file_size: metadata.len(),
        blake3sum,
        mtime,
        atime,
        unix_mode: metadata.permissions().mode(),
        uid: metadata.uid(),
        gid: metadata.gid(),
    })
}

/// Computes blake3 hash of a file
pub fn blake3sum(filename: &std::path::Path) -> Result<String, std::io::Error> {
    let mut hasher = Hasher::new();
    let mut file = std::fs::File::open(filename)?;
    std::io::copy(&mut file, &mut hasher)?;
    let digest = hasher.finalize();
    let mut hex_string: String = "".into();
    hex_string.push_str(&digest.to_hex());
    Ok(hex_string)
}
