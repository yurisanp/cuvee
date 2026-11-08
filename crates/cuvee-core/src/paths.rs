//! Where the runtime lives: QEMU with reims-vgpu, reims-ctl and reims-util.
//!
//! Two kinds of install, looked up in this order:
//!
//! - a system one, from a distribution package (the Arch PKGBUILD, the .deb or .rpm):
//!   QEMU in `/usr/lib/cuvee/qemu`, `reims-ctl` and `reims-util` on the PATH. The package
//!   writes `/usr/lib/cuvee/packaged` with its kind (`pacman`, `deb`, `rpm`), and then
//!   `cuvee install` and `cuvee update` leave the runtime to the package manager;
//! - a user one, from `cuvee install`: `~/.local/share/cuvee/runtime/<tag>/` holding
//!   `qemu/`, `bin/reims-ctl` and `bin/reims-util`, with `runtime/current` linking the one in
//!   use. A new install goes beside the old one and moves the link, so a running VM keeps the
//!   QEMU it started with.
//!
//! `REIMS_CTL_QEMU` and `REIMS_CTL_UTIL`, reims-ctl's own variables, override both.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The system install's prefix.
pub const SYSTEM_LIB: &str = "/usr/lib/cuvee";

fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

/// `~/.local/share/cuvee` (or `$XDG_DATA_HOME/cuvee`).
pub fn data_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("cuvee")
}

/// `~/.cache/cuvee` (or `$XDG_CACHE_HOME/cuvee`): downloads before they are unpacked.
pub fn cache_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join("cuvee")
}

/// Where `cuvee install` puts the `cuvee` and `cuvee-gui` binaries: `~/.local/bin`.
pub fn user_bin_dir() -> PathBuf {
    home().join(".local/bin")
}

/// The directory holding the user's runtimes, one per release tag, and `current`.
pub fn runtimes_dir() -> PathBuf {
    data_dir().join("runtime")
}

/// The user runtime in use (`runtime/current`), if one is installed.
pub fn user_runtime() -> Option<PathBuf> {
    let current = runtimes_dir().join("current");
    current
        .join("qemu/bin/qemu-system-x86_64")
        .is_file()
        .then_some(current)
}

/// What packaged the system install (`pacman`, `deb`, `rpm`), if a package did.
pub fn packaged() -> Option<String> {
    packaged_in(Path::new(SYSTEM_LIB))
}

fn packaged_in(lib: &Path) -> Option<String> {
    let s = std::fs::read_to_string(lib.join("packaged")).ok()?;
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// The first `bin` in the PATH.
pub fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(bin))
        .find(|p| p.is_file())
}

/// QEMU with reims-vgpu: `$REIMS_CTL_QEMU`, the system's, the user's.
pub fn qemu_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("REIMS_CTL_QEMU").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(p));
    }
    let system = Path::new(SYSTEM_LIB).join("qemu/bin/qemu-system-x86_64");
    if system.is_file() {
        return Some(system);
    }
    user_runtime().map(|r| r.join("qemu/bin/qemu-system-x86_64"))
}

/// reims-ctl: the system's on the PATH, else the user runtime's.
pub fn reims_ctl() -> Option<PathBuf> {
    which("reims-ctl").or_else(|| {
        user_runtime()
            .map(|r| r.join("bin/reims-ctl"))
            .filter(|p| p.is_file())
    })
}

/// reims-util: `$REIMS_CTL_UTIL`, the one beside reims-ctl, the PATH's.
pub fn reims_util() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("REIMS_CTL_UTIL").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(p));
    }
    reims_ctl()
        .and_then(|c| Some(c.parent()?.join("reims-util")))
        .filter(|p| p.is_file())
        .or_else(|| which("reims-util"))
}

/// A `reims-ctl` command that finds this install's QEMU and reims-util. Runs `reims-ctl` from
/// the PATH when none is installed, so the error is the usual "not found".
pub fn reims_ctl_command() -> Command {
    let mut c = Command::new(reims_ctl().unwrap_or_else(|| PathBuf::from("reims-ctl")));
    if std::env::var_os("REIMS_CTL_QEMU").is_none() {
        if let Some(q) = qemu_binary() {
            c.env("REIMS_CTL_QEMU", q);
        }
    }
    if std::env::var_os("REIMS_CTL_UTIL").is_none() {
        if let Some(u) = reims_util() {
            c.env("REIMS_CTL_UTIL", u);
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_reads_the_marker() {
        let dir = std::env::temp_dir().join(format!("cuvee-paths-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(packaged_in(&dir), None);
        std::fs::write(dir.join("packaged"), "deb\n").unwrap();
        assert_eq!(packaged_in(&dir).as_deref(), Some("deb"));
        std::fs::write(dir.join("packaged"), "  \n").unwrap();
        assert_eq!(packaged_in(&dir), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn runtime_lives_in_the_data_dir() {
        assert!(runtimes_dir().ends_with("cuvee/runtime"));
    }
}
