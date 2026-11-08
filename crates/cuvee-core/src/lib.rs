//! Cuvée's core: what the CLI and the desktop app share.
//!
//! - [`host`]: the checks `cuvee doctor` runs (KVM, udmabuf, GPU, session, IOMMU, install).
//! - [`macos`]: the catalog of macOS versions a VM can run, with what to expect from each.
//! - [`vm`]: the VMs reims-ctl knows, read through `reims-ctl list --json`.
//! - [`i18n`]: Fluent translations; every user-facing string goes through [`i18n::Locale`].
//! - [`paths`]: where QEMU, reims-ctl and reims-util are (system or user install).
//! - [`distro`]: the distribution, its package manager and the packages the runtime needs.
//! - [`release`]: downloading and checking a release's files from GitHub.
//! - [`signature`]: where release signatures are checked.
//! - [`settings`]: the app's own settings (language, reduced motion) in the XDG config dir.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

pub mod distro;
pub mod host;
pub mod i18n;
pub mod macos;
pub mod paths;
pub mod release;
pub mod settings;
pub mod signature;
pub mod vm;
