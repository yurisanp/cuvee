//! Cuvée's core: what the CLI and the desktop app share.
//!
//! - [`host`]: the checks `cuvee doctor` runs (KVM, udmabuf, GPU, session, IOMMU, install).
//! - [`macos`]: the catalog of macOS versions a VM can run, with what to expect from each.
//! - [`vm`]: the VMs reims-ctl knows, read through `reims-ctl list --json`.
//! - [`i18n`]: Fluent translations; every user-facing string goes through [`i18n::Locale`].
//! - [`report`]: the text file `cuvee report` writes for bug reports.
//! - [`settings`]: the app's own settings (language, reduced motion) in the XDG config dir.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

pub mod host;
pub mod i18n;
pub mod macos;
pub mod report;
pub mod settings;
pub mod vm;
