//! Signatures of release files (minisign).
//!
//! Not checked yet: releases are not signed until the project's key exists. [`verify`] is
//! where [`crate::release::Fetcher`] asks, after the SHA-256 matched, so that turning
//! signatures on changes this file alone.

use anyhow::Result;
use std::path::Path;

/// Checks `file` against its `.minisig`, `sig`, when the release has one. Accepts every file
/// for now.
pub fn verify(_file: &Path, _sig: Option<&Path>) -> Result<()> {
    Ok(())
}
