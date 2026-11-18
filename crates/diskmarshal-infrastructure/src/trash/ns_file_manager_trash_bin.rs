//! Trash adapter built on `NSFileManager`.

use std::io;
use std::path::Path;

use ::trash::TrashContext;
use ::trash::macos::{DeleteMethod, TrashContextExtMacos as _};
use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::TrashBin;

/// Moves items to the Trash through `NSFileManager`.
///
/// The `trash` crate defaults to scripting Finder, which can ask the user for
/// automation permission. `NSFileManager` needs none.
pub struct NsFileManagerTrashBin {
    context: TrashContext,
}

impl NsFileManagerTrashBin {
    /// Creates the adapter.
    #[must_use]
    pub fn new() -> Self {
        let mut context = TrashContext::default();
        context.set_delete_method(DeleteMethod::NsFileManager);
        Self { context }
    }
}

impl Default for NsFileManagerTrashBin {
    fn default() -> Self {
        Self::new()
    }
}

impl TrashBin for NsFileManagerTrashBin {
    fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError> {
        self.context
            .delete(path)
            .map_err(|error| ApplicationError::Io {
                path: path.to_path_buf(),
                source: io::Error::other(error.to_string()),
            })
    }
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::TrashBin;
    use diskmarshal_test_support::TempTree;

    use super::NsFileManagerTrashBin;

    /// Puts a real file in the user's Trash, so it only runs on request:
    /// `cargo test -p diskmarshal-infrastructure -- --ignored`.
    #[test]
    #[ignore = "moves a file into the real Trash"]
    fn moves_a_file_out_of_its_folder() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.file("diskmarshal-trash-test.txt", 10).unwrap();

        NsFileManagerTrashBin::new().move_to_trash(&file).unwrap();

        assert!(!file.exists());
    }
}
