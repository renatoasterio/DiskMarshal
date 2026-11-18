//! Who is allowed to remove something.

/// The rights a removal needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivilegeScope {
    /// The logged-in user can remove it.
    User,
    /// macOS must authenticate an administrator first.
    Administrator,
}
