//! The technology a code project is built with.

/// A family of tools that produces recognizable build folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ecosystem {
    /// JavaScript and TypeScript projects with a `package.json`.
    Node,
    /// .NET projects and solutions.
    DotNet,
    /// Rust projects with a `Cargo.toml`.
    Rust,
    /// Java and Kotlin projects built with Maven or Gradle.
    Jvm,
    /// Python projects.
    Python,
    /// Swift packages and CocoaPods projects.
    Swift,
    /// Flutter and Dart projects.
    Flutter,
    /// Go modules.
    Go,
    /// PHP projects managed by Composer.
    Php,
    /// Terraform configurations.
    Terraform,
}

impl Ecosystem {
    /// Every ecosystem, in the order they are offered as filters.
    pub const ALL: [Self; 10] = [
        Self::Node,
        Self::DotNet,
        Self::Rust,
        Self::Jvm,
        Self::Python,
        Self::Swift,
        Self::Flutter,
        Self::Go,
        Self::Php,
        Self::Terraform,
    ];

    /// The name developers know it by. It is the same in every language.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Node => "Node",
            Self::DotNet => ".NET",
            Self::Rust => "Rust",
            Self::Jvm => "Java/Kotlin",
            Self::Python => "Python",
            Self::Swift => "Swift",
            Self::Flutter => "Flutter",
            Self::Go => "Go",
            Self::Php => "PHP",
            Self::Terraform => "Terraform",
        }
    }
}
