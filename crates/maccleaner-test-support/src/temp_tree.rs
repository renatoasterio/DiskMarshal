//! A throwaway folder tree on the real file system.

use std::fs;
use std::io;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// A temporary folder that is deleted when the value is dropped.
///
/// Paths given to its methods are relative to the temporary root; missing
/// parent folders are created as needed.
#[derive(Debug)]
pub struct TempTree {
    directory: TempDir,
}

impl TempTree {
    /// Creates an empty temporary folder.
    ///
    /// # Errors
    ///
    /// Fails when the temporary folder cannot be created.
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            directory: TempDir::new()?,
        })
    }

    /// The temporary folder itself.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.directory.path()
    }

    /// Resolves a relative path against the temporary root.
    #[must_use]
    pub fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }

    /// Creates a folder.
    ///
    /// # Errors
    ///
    /// Fails when the folder cannot be created.
    pub fn directory(&self, relative: &str) -> io::Result<PathBuf> {
        let path = self.path(relative);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Creates a file filled with `size` bytes.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    pub fn file(&self, relative: &str, size: usize) -> io::Result<PathBuf> {
        let path = self.path(relative);
        Self::create_parent(&path)?;
        fs::write(&path, vec![0xA5_u8; size])?;
        Ok(path)
    }

    /// Creates a symbolic link at `relative` that points to `target`.
    ///
    /// # Errors
    ///
    /// Fails when the link cannot be created.
    pub fn symlink(&self, target: &Path, relative: &str) -> io::Result<PathBuf> {
        let path = self.path(relative);
        Self::create_parent(&path)?;
        unix_fs::symlink(target, &path)?;
        Ok(path)
    }

    /// Creates a second name (hard link) for an existing file.
    ///
    /// # Errors
    ///
    /// Fails when the link cannot be created.
    pub fn hard_link(&self, existing: &str, relative: &str) -> io::Result<PathBuf> {
        let path = self.path(relative);
        Self::create_parent(&path)?;
        fs::hard_link(self.path(existing), &path)?;
        Ok(path)
    }

    fn create_parent(path: &Path) -> io::Result<()> {
        match path.parent() {
            Some(parent) => fs::create_dir_all(parent),
            None => Ok(()),
        }
    }
}
