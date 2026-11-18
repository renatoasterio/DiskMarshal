//! Telling which program a process belongs to.

use std::path::{Component, Path};

/// Extension of an app bundle.
const APP_EXTENSION: &str = "app";

/// The name people know a running process by.
///
/// A browser or an editor runs as many processes with names of their own
/// (`Editor Helper (Renderer)`), all inside its bundle. The outermost `.app`
/// folder on the path of the executable names the program they belong to.
/// Outside a bundle the file name of the executable is used, and when macOS
/// does not say where the process was started from, `process_name` is all
/// there is.
#[must_use]
pub fn program_name(executable: Option<&Path>, process_name: &str) -> String {
    let Some(executable) = executable else {
        return process_name.to_owned();
    };
    let bundle = executable
        .components()
        .find_map(|component| match component {
            Component::Normal(name) => {
                let name = Path::new(name);
                name.extension()
                    .is_some_and(|extension| extension == APP_EXTENSION)
                    .then(|| name.file_stem())
                    .flatten()
            }
            _ => None,
        });
    bundle.or_else(|| executable.file_name()).map_or_else(
        || process_name.to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::program_name;

    #[test]
    fn a_helper_inside_a_bundle_is_named_after_the_outermost_app() {
        let helper = Path::new(
            "/Applications/Editor.app/Contents/Frameworks/Editor Helper (Renderer).app/Contents/MacOS/Editor Helper (Renderer)",
        );
        assert_eq!(program_name(Some(helper), "Editor Helper (R"), "Editor");
    }

    #[test]
    fn a_program_outside_a_bundle_is_named_after_its_file() {
        let daemon = Path::new("/usr/sbin/mDNSResponder");
        assert_eq!(program_name(Some(daemon), "mDNSRespond"), "mDNSResponder");
    }

    #[test]
    fn without_an_executable_the_process_name_is_all_there_is() {
        assert_eq!(program_name(None, "apsd"), "apsd");
        assert_eq!(program_name(Some(Path::new("/")), "launchd"), "launchd");
    }
}
