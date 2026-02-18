use std::io;
use std::path::Path;

/// Recursively copies all files and subdirectories from `src` to `dst`.
///
/// Creates `dst` directory and all necessary parent directories.
/// Overwrites existing files with the same name in `dst`.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Errors
///
/// Returns `io::Error` if directory creation or file copy operations fail.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
