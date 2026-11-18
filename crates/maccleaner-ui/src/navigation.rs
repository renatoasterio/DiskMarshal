//! The screens of the app and how the user reaches them.

use egui::Key;

use crate::components::Icon;
use crate::i18n::pt_br;

/// One of the screens of the app.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Screen {
    /// Overview of the disk, memory and what can be recovered.
    #[default]
    Dashboard,
    /// Caches, temporary files and logs.
    Cleanup,
    /// Installed apps and what uninstalling one removes.
    Apps,
    /// Files with identical content.
    Duplicates,
    /// Identical and visually similar photos.
    Photos,
    /// Build and dependency folders of code projects.
    Developer,
    /// What Docker holds that nothing uses.
    Docker,
    /// Treemap of what uses the disk.
    DiskMap,
    /// Live processor, memory, disk and sensor readings.
    Monitor,
}

impl Screen {
    /// Every screen, in the order of the navigation rail.
    pub const ALL: [Self; 9] = [
        Self::Dashboard,
        Self::Cleanup,
        Self::Apps,
        Self::Duplicates,
        Self::Photos,
        Self::Developer,
        Self::Docker,
        Self::DiskMap,
        Self::Monitor,
    ];

    /// Finds a screen by its identifier, such as `disk_map`.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|screen| screen.slug() == slug)
    }

    /// Whether the screen reads the user's own folders, where macOS hides
    /// content from apps without Full Disk Access.
    #[must_use]
    pub const fn reads_personal_folders(self) -> bool {
        !matches!(self, Self::Dashboard | Self::Docker | Self::Monitor)
    }

    /// Identifier of the screen, stable across languages.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Cleanup => "cleanup",
            Self::Apps => "apps",
            Self::Duplicates => "duplicates",
            Self::Photos => "photos",
            Self::Developer => "developer",
            Self::Docker => "docker",
            Self::DiskMap => "disk_map",
            Self::Monitor => "monitor",
        }
    }

    /// Name shown in the navigation rail.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dashboard => pt_br::SCREEN_DASHBOARD,
            Self::Cleanup => pt_br::SCREEN_CLEANUP,
            Self::Apps => pt_br::SCREEN_APPS,
            Self::Duplicates => pt_br::SCREEN_DUPLICATES,
            Self::Photos => pt_br::SCREEN_PHOTOS,
            Self::Developer => pt_br::SCREEN_DEVELOPER,
            Self::Docker => pt_br::SCREEN_DOCKER,
            Self::DiskMap => pt_br::SCREEN_DISK_MAP,
            Self::Monitor => pt_br::SCREEN_MONITOR,
        }
    }

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Dashboard => pt_br::TITLE_DASHBOARD,
            Self::Cleanup => pt_br::TITLE_CLEANUP,
            Self::Apps => pt_br::TITLE_APPS,
            Self::Duplicates => pt_br::TITLE_DUPLICATES,
            Self::Photos => pt_br::TITLE_PHOTOS,
            Self::Developer => pt_br::TITLE_DEVELOPER,
            Self::Docker => pt_br::TITLE_DOCKER,
            Self::DiskMap => pt_br::TITLE_DISK_MAP,
            Self::Monitor => pt_br::TITLE_MONITOR,
        }
    }

    pub(crate) const fn subtitle(self) -> &'static str {
        match self {
            Self::Dashboard => pt_br::SUBTITLE_DASHBOARD,
            Self::Cleanup => pt_br::SUBTITLE_CLEANUP,
            Self::Apps => pt_br::SUBTITLE_APPS,
            Self::Duplicates => pt_br::SUBTITLE_DUPLICATES,
            Self::Photos => pt_br::SUBTITLE_PHOTOS,
            Self::Developer => pt_br::SUBTITLE_DEVELOPER,
            Self::Docker => pt_br::SUBTITLE_DOCKER,
            Self::DiskMap => pt_br::SUBTITLE_DISK_MAP,
            Self::Monitor => pt_br::SUBTITLE_MONITOR,
        }
    }

    pub(crate) const fn icon(self) -> Icon {
        match self {
            Self::Dashboard => Icon::Dashboard,
            Self::Cleanup => Icon::Cleanup,
            Self::Apps => Icon::Apps,
            Self::Duplicates => Icon::Duplicates,
            Self::Photos => Icon::Photos,
            Self::Developer => Icon::Developer,
            Self::Docker => Icon::Docker,
            Self::DiskMap => Icon::DiskMap,
            Self::Monitor => Icon::Monitor,
        }
    }

    /// The number key that opens the screen, and how it is shown in the rail.
    pub(crate) const fn shortcut(self) -> (Key, &'static str) {
        match self {
            Self::Dashboard => (Key::Num1, "1"),
            Self::Cleanup => (Key::Num2, "2"),
            Self::Apps => (Key::Num3, "3"),
            Self::Duplicates => (Key::Num4, "4"),
            Self::Photos => (Key::Num5, "5"),
            Self::Developer => (Key::Num6, "6"),
            Self::Docker => (Key::Num7, "7"),
            Self::DiskMap => (Key::Num8, "8"),
            Self::Monitor => (Key::Num9, "9"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Screen;

    #[test]
    fn every_screen_is_found_by_its_slug() {
        for screen in Screen::ALL {
            assert_eq!(Screen::from_slug(screen.slug()), Some(screen));
        }
        assert_eq!(Screen::from_slug("nope"), None);
    }

    #[test]
    fn no_two_screens_share_a_shortcut() {
        let keys: HashSet<_> = Screen::ALL
            .iter()
            .map(|screen| screen.shortcut().0)
            .collect();
        assert_eq!(keys.len(), Screen::ALL.len());
    }
}
