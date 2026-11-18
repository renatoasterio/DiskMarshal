//! A fingerprint of a file's content.

/// A 256-bit digest. Two files with the same full-content digest are treated
/// as identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentHash(pub [u8; 32]);
