//! What an item that belongs to an app is.

/// The kind of item an uninstall can remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeftoverKind {
    /// The app bundle itself.
    Application,
    /// Data the app keeps for itself, such as documents and databases.
    Support,
    /// Data the app can download or compute again.
    Caches,
    /// The settings the user chose in the app.
    Preferences,
    /// The private folder macOS gives a sandboxed app, or one it shares
    /// with other apps of the same vendor.
    Container,
    /// The windows the app had open, kept to reopen them.
    SavedState,
    /// Cookies and other data of the web pages the app showed.
    WebData,
    /// Logs the app wrote.
    Logs,
    /// A program macOS starts for the app, at login or at start-up.
    LaunchItem,
    /// A program the app installed to work with administrator rights.
    Helper,
}
