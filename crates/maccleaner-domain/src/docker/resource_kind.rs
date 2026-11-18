//! The kinds of thing Docker keeps on disk.

/// What a Docker resource is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    /// Layers kept to make the next build faster. Removed as a whole.
    BuildCache,
    /// An image: the files a container starts from.
    Image,
    /// A container: an image plus what was written while it ran.
    Container,
    /// A volume: data kept outside any container, such as a database.
    Volume,
}

impl ResourceKind {
    /// Every kind, in the order they are shown.
    pub const ALL: [Self; 4] = [Self::BuildCache, Self::Image, Self::Container, Self::Volume];

    /// Identifier of the kind, stable across languages.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::BuildCache => "build-cache",
            Self::Image => "image",
            Self::Container => "container",
            Self::Volume => "volume",
        }
    }
}
