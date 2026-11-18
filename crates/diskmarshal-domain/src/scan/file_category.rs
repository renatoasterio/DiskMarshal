//! What kind of content an item holds, judged from its name.

use crate::scan::EntryKind;

/// A broad kind of content, used to color the disk map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    /// macOS itself, apps and their support files.
    System,
    /// Source code and what building it produces.
    Code,
    /// Pictures.
    Photo,
    /// Video and audio.
    Media,
    /// Text, spreadsheets, presentations, PDFs.
    Document,
    /// Compressed archives, disk images and installers.
    Archive,
    /// Anything the name does not explain.
    Other,
}

impl FileCategory {
    /// Every category, in the order of the map's legend.
    pub const ALL: [Self; 7] = [
        Self::System,
        Self::Code,
        Self::Photo,
        Self::Media,
        Self::Document,
        Self::Archive,
        Self::Other,
    ];

    /// Classifies an item from its name and kind. The comparison ignores case.
    #[must_use]
    pub fn of(name: &str, kind: EntryKind) -> Self {
        let lowercase = name.to_lowercase();
        let extension = lowercase
            .rsplit_once('.')
            .map_or("", |(_, extension)| extension);
        match kind {
            EntryKind::Package if extension == "photoslibrary" => Self::Photo,
            EntryKind::Package => Self::System,
            EntryKind::Directory => Self::of_folder(&lowercase),
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => {
                Self::of_extension(extension)
            }
        }
    }

    fn of_folder(name: &str) -> Self {
        match name {
            "library"
            | "applications"
            | "system"
            | "private"
            | "usr"
            | "opt"
            | "bin"
            | "sbin"
            | "containers"
            | "application support"
            | "caches" => Self::System,
            "pictures" | "photos" | "imagens" | "fotos" => Self::Photo,
            "movies" | "music" | "filmes" | "música" => Self::Media,
            "documents" | "desktop" | "documentos" | "mesa" | "mobile documents" => Self::Document,
            "downloads" => Self::Archive,
            "node_modules" | "target" | "src" | ".git" | "developer" | "projects" | "projetos"
            | "repository" | "repositories" | "code" | "workspace" | "build" | "dist"
            | "vendor" | "deriveddata" | ".venv" | "venv" => Self::Code,
            _ => Self::Other,
        }
    }

    fn of_extension(extension: &str) -> Self {
        match extension {
            "jpg" | "jpeg" | "png" | "heic" | "heif" | "gif" | "tiff" | "tif" | "bmp" | "webp"
            | "raw" | "cr2" | "cr3" | "nef" | "arw" | "dng" | "psd" | "svg" => Self::Photo,
            "mov" | "mp4" | "m4v" | "mkv" | "avi" | "webm" | "mp3" | "m4a" | "wav" | "flac"
            | "aac" | "aiff" | "ogg" => Self::Media,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "pages" | "numbers"
            | "key" | "txt" | "md" | "rtf" | "csv" | "epub" | "odt" => Self::Document,
            "zip" | "dmg" | "pkg" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" | "iso"
            | "ipa" | "xip" => Self::Archive,
            "rs" | "js" | "ts" | "tsx" | "jsx" | "py" | "java" | "kt" | "swift" | "c" | "h"
            | "cpp" | "hpp" | "cs" | "go" | "rb" | "php" | "html" | "css" | "json" | "toml"
            | "yaml" | "yml" | "xml" | "sh" | "sql" | "o" | "a" | "rlib" | "rmeta" | "class"
            | "jar" | "dll" | "pdb" | "wasm" | "map" | "lock" => Self::Code,
            "dylib" | "so" | "plist" | "db" | "sqlite" | "log" | "cache" => Self::System,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileCategory;
    use crate::scan::EntryKind;

    #[test]
    fn classifies_files_by_extension_ignoring_case() {
        assert_eq!(
            FileCategory::of("IMG_0412.HEIC", EntryKind::File),
            FileCategory::Photo
        );
        assert_eq!(
            FileCategory::of("reuniao.mov", EntryKind::File),
            FileCategory::Media
        );
        assert_eq!(
            FileCategory::of("contrato.pdf", EntryKind::File),
            FileCategory::Document
        );
        assert_eq!(
            FileCategory::of("backup.tar.gz", EntryKind::File),
            FileCategory::Archive
        );
        assert_eq!(
            FileCategory::of("main.rs", EntryKind::File),
            FileCategory::Code
        );
        assert_eq!(
            FileCategory::of("sem-extensao", EntryKind::File),
            FileCategory::Other
        );
    }

    #[test]
    fn classifies_well_known_folders_by_name() {
        assert_eq!(
            FileCategory::of("Library", EntryKind::Directory),
            FileCategory::System
        );
        assert_eq!(
            FileCategory::of("node_modules", EntryKind::Directory),
            FileCategory::Code
        );
        assert_eq!(
            FileCategory::of("Pictures", EntryKind::Directory),
            FileCategory::Photo
        );
        assert_eq!(
            FileCategory::of("viagem-2024", EntryKind::Directory),
            FileCategory::Other
        );
    }

    #[test]
    fn a_photos_library_is_photos_and_other_packages_are_system() {
        assert_eq!(
            FileCategory::of("Photos Library.photoslibrary", EntryKind::Package),
            FileCategory::Photo
        );
        assert_eq!(
            FileCategory::of("Xcode.app", EntryKind::Package),
            FileCategory::System
        );
    }
}
