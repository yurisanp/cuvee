//! `cuvee report`: one text file with what a bug report needs about the host.
//!
//! The file is plain text so it can be pasted into a GitHub issue. It holds the
//! distro, kernel, CPU, GPUs and their drivers, the session, the installed
//! versions, the result of every host check, `vulkaninfo --summary` and the VMs
//! reims-ctl knows. With a VM name it also runs `reims-ctl report NAME`, which
//! zips that VM's logs and settings, and names the zip.
//!
//! Nothing personal goes in: the home directory becomes `~`, the user name
//! `<user>` and the host name `<host>`. The kernel command line is reduced to
//! the IOMMU and VFIO parameters.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

use crate::host::{self, GpuVendor, Level, Session};
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Lines kept from long command outputs (vulkaninfo, reims-ctl doctor).
const MAX_LINES: usize = 80;

/// Builds the report. `vm` also runs `reims-ctl report VM`.
pub fn collect(vm: Option<&str>) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# Cuvée report");
    let _ = writeln!(s);
    let _ = writeln!(s, "cuvee-core {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(
        s,
        "date: {}",
        run(&["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"]).trim()
    );

    section(&mut s, "System");
    let os = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let _ = writeln!(
        s,
        "distro: {} (id={}, version={})",
        os_release(&os, "PRETTY_NAME").unwrap_or("unknown"),
        os_release(&os, "ID").unwrap_or("?"),
        os_release(&os, "VERSION_ID").unwrap_or("?"),
    );
    let _ = writeln!(s, "kernel: {}", read_trim("/proc/sys/kernel/osrelease"));
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let _ = writeln!(
        s,
        "cpu: {} ({} threads)",
        cpu_model(&cpuinfo).unwrap_or("unknown"),
        crate::vm::host_cpus()
    );
    let _ = writeln!(s, "memory: {} GiB", crate::vm::host_memory_gib());
    let _ = writeln!(s, "installed by: {}", installed_by());
    let _ = writeln!(s, "cmdline (iommu/vfio only): {}", iommu_cmdline());
    let modules = std::fs::read_to_string("/proc/modules").unwrap_or_default();
    let _ = writeln!(
        s,
        "modules: {}",
        [
            "kvm_intel",
            "kvm_amd",
            "udmabuf",
            "vfio_pci",
            "nvidia",
            "amdgpu",
            "i915",
            "xe"
        ]
        .iter()
        .filter(|m| module_loaded(&modules, m))
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
    );
    let _ = writeln!(
        s,
        "vm.max_map_count: {}",
        read_trim("/proc/sys/vm/max_map_count")
    );
    let _ = writeln!(s, "in kvm group: {}", in_group("kvm"));
    for dev in ["/dev/kvm", "/dev/udmabuf"] {
        let _ = writeln!(s, "{dev}: {}", device_line(Path::new(dev)));
    }

    section(&mut s, "Graphics");
    let _ = writeln!(
        s,
        "session: {} (XDG_SESSION_TYPE={}, desktop={})",
        match host::session() {
            Session::Wayland => "wayland",
            Session::X11 => "x11",
            Session::None => "none",
        },
        std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
    );
    for gpu in gpus() {
        let _ = writeln!(s, "gpu: {gpu}");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "vulkaninfo --summary:");
    s.push_str(&indent(&head(
        &run(&["vulkaninfo", "--summary"]),
        MAX_LINES,
    )));

    section(&mut s, "Versions");
    for cmd in [
        &["reims-ctl", "--version"][..],
        &["reims-util", "--version"],
        &[host::QEMU_BIN, "--version"],
    ] {
        let out = run(cmd);
        let _ = writeln!(
            s,
            "{}: {}",
            cmd[0],
            out.lines().next().unwrap_or("not found").trim()
        );
    }

    section(&mut s, "Checks");
    let report = host::inspect();
    for c in &report.checks {
        let level = match c.level {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Bad => "BAD ",
        };
        let _ = writeln!(s, "{level} {:?}", c.id);
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "reims-ctl doctor:");
    s.push_str(&indent(&head(&run(&["reims-ctl", "doctor"]), MAX_LINES)));

    section(&mut s, "VMs");
    match crate::vm::list() {
        Ok(vms) if vms.is_empty() => s.push_str("none\n"),
        Ok(vms) => {
            for v in vms {
                let _ = writeln!(s, "{} ({:?})", v.name, v.state);
            }
        }
        Err(e) => {
            let _ = writeln!(s, "reims-ctl list failed: {e:#}");
        }
    }
    if let Some(name) = vm {
        let _ = writeln!(s);
        let _ = writeln!(s, "reims-ctl report {name}:");
        s.push_str(&indent(&head(
            &run(&["reims-ctl", "report", name]),
            MAX_LINES,
        )));
    }

    redact(&s, &Identity::current())
}

/// Writes the report to the Downloads folder (or home) and returns its path.
pub fn write(vm: Option<&str>) -> Result<PathBuf> {
    let text = collect(vm);
    let stamp = run(&["date", "+%Y%m%d-%H%M%S"]);
    let stamp = stamp.trim();
    let path = output_dir().join(format!(
        "cuvee-report-{}.txt",
        if stamp.is_empty() { "now" } else { stamp }
    ));
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

fn section(s: &mut String, name: &str) {
    let _ = writeln!(s);
    let _ = writeln!(s, "## {name}");
    let _ = writeln!(s);
}

/// Runs a command and returns stdout and stderr together, or "" if it cannot start.
fn run(cmd: &[&str]) -> String {
    match Command::new(cmd[0]).args(&cmd[1..]).output() {
        Ok(o) => {
            let mut out = String::from_utf8_lossy(&o.stdout).into_owned();
            out.push_str(&String::from_utf8_lossy(&o.stderr));
            out
        }
        Err(_) => String::new(),
    }
}

fn read_trim(path: &str) -> String {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "?".into())
}

fn head(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return "(not available)\n".into();
    }
    let mut out = lines[..lines.len().min(n)].join("\n");
    out.push('\n');
    if lines.len() > n {
        let _ = writeln!(out, "… {} more lines", lines.len() - n);
    }
    out
}

fn indent(text: &str) -> String {
    text.lines().map(|l| format!("    {l}\n")).collect()
}

/// A value from /etc/os-release, without quotes.
pub fn os_release<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|l| {
        let v = l.strip_prefix(key)?.strip_prefix('=')?;
        Some(v.trim().trim_matches('"').trim_matches('\''))
    })
}

fn cpu_model(cpuinfo: &str) -> Option<&str> {
    cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
}

fn module_loaded(modules: &str, name: &str) -> bool {
    modules
        .lines()
        .any(|l| l.split_whitespace().next() == Some(name))
}

/// How Cuvée got here: the native packages write /usr/lib/cuvee/packaged.
fn installed_by() -> String {
    match std::fs::read_to_string("/usr/lib/cuvee/packaged") {
        Ok(s) => s.trim().to_string(),
        Err(_) if Path::new(host::QEMU_BIN).exists() => "pacman or manual".into(),
        Err(_) => "user install or not installed".into(),
    }
}

/// The kernel parameters that matter for GPU passthrough; the rest (root, UUIDs) stays out.
fn iommu_cmdline() -> String {
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();
    let kept: Vec<&str> = cmdline
        .split_whitespace()
        .filter(|p| {
            let key = p.split('=').next().unwrap_or("");
            key.contains("iommu") || key.starts_with("vfio") || key.contains("pcie_acs")
        })
        .collect();
    if kept.is_empty() {
        "(none)".into()
    } else {
        kept.join(" ")
    }
}

fn in_group(group: &str) -> bool {
    run(&["id", "-nG"]).split_whitespace().any(|g| g == group)
}

fn device_line(path: &Path) -> String {
    use std::os::unix::fs::MetadataExt;
    match std::fs::metadata(path) {
        Ok(m) => {
            let group = run(&["stat", "-c", "%G", &path.to_string_lossy()]);
            format!("mode {:o}, group {}", m.mode() & 0o777, group.trim())
        }
        Err(_) => "missing".into(),
    }
}

/// Each GPU: vendor, PCI ids and the kernel driver bound to it.
fn gpus() -> Vec<String> {
    let mut out = Vec::new();
    let Ok(cards) = std::fs::read_dir("/sys/class/drm") else {
        return out;
    };
    let mut names: Vec<_> = cards
        .flatten()
        .map(|c| c.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("card") && !n.contains('-'))
        .collect();
    names.sort();
    for name in names {
        let dev = PathBuf::from("/sys/class/drm").join(&name).join("device");
        let vendor = read_trim(&dev.join("vendor").to_string_lossy());
        let device = read_trim(&dev.join("device").to_string_lossy());
        let driver = std::fs::read_link(dev.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "none".into());
        let label = match GpuVendor::from_pci(&vendor) {
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Other => "other",
        };
        out.push(format!(
            "{name}: {label} {vendor}:{device}, driver {driver}"
        ));
    }
    out
}

fn output_dir() -> PathBuf {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
    let from_xdg = run(&["xdg-user-dir", "DOWNLOAD"]);
    let from_xdg = from_xdg.trim();
    if !from_xdg.is_empty() && Path::new(from_xdg).is_dir() && Path::new(from_xdg) != home {
        return PathBuf::from(from_xdg);
    }
    for d in ["Downloads", "Transferências"] {
        let p = home.join(d);
        if p.is_dir() {
            return p;
        }
    }
    home
}

/// What is replaced in the report so it can be shared.
pub struct Identity {
    pub home: String,
    pub user: String,
    pub host: String,
}

impl Identity {
    fn current() -> Self {
        Self {
            home: std::env::var("HOME").unwrap_or_default(),
            user: std::env::var("USER")
                .ok()
                .filter(|u| !u.is_empty())
                .unwrap_or_else(|| run(&["id", "-un"]).trim().to_string()),
            host: read_trim("/proc/sys/kernel/hostname"),
        }
    }
}

/// Replaces the home directory, the user name and the host name.
pub fn redact(text: &str, id: &Identity) -> String {
    let mut out = text.to_string();
    // Home first: it usually contains the user name.
    if id.home.len() > 1 {
        out = out.replace(&id.home, "~");
    }
    // Short names ("a", "pc") would hit ordinary words; three letters is the floor.
    if id.user.len() >= 3 && id.user != "root" {
        out = out.replace(&id.user, "<user>");
    }
    if id.host.len() >= 3 && id.host != "?" && id.host != "localhost" {
        out = out.replace(&id.host, "<host>");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_os_release() {
        let text = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\nVERSION_ID='24.04'\n";
        assert_eq!(os_release(text, "PRETTY_NAME"), Some("Arch Linux"));
        assert_eq!(os_release(text, "ID"), Some("arch"));
        assert_eq!(os_release(text, "VERSION_ID"), Some("24.04"));
        assert_eq!(os_release(text, "NAME"), Some("Arch Linux"));
        assert_eq!(os_release(text, "BUILD_ID"), None);
    }

    #[test]
    fn redacts_identity() {
        let id = Identity {
            home: "/home/yuri".into(),
            user: "yuri".into(),
            host: "desktop-yuri".into(),
        };
        let text = "dir /home/yuri/.local/share/reims-ctl/vms/mac\nuser yuri on desktop-yuri\n";
        let out = redact(text, &id);
        assert!(!out.contains("yuri"), "{out}");
        assert!(out.contains("~/.local/share/reims-ctl/vms/mac"));
    }

    #[test]
    fn short_names_are_left_alone() {
        let id = Identity {
            home: "/".into(),
            user: "a".into(),
            host: "pc".into(),
        };
        assert_eq!(redact("a pc /x", &id), "a pc /x");
    }

    #[test]
    fn head_cuts_long_output() {
        let text = (0..100).map(|i| format!("{i}\n")).collect::<String>();
        let out = head(&text, 3);
        assert_eq!(out, "0\n1\n2\n… 97 more lines\n");
        assert_eq!(head("", 3), "(not available)\n");
    }

    #[test]
    fn finds_modules() {
        let m = "udmabuf 20480 0 - Live 0x0\nkvm_amd 200000 0 - Live 0x0\n";
        assert!(module_loaded(m, "udmabuf"));
        assert!(module_loaded(m, "kvm_amd"));
        assert!(!module_loaded(m, "kvm"));
    }

    #[test]
    fn report_has_every_section() {
        let r = collect(None);
        for s in [
            "## System",
            "## Graphics",
            "## Versions",
            "## Checks",
            "## VMs",
        ] {
            assert!(r.contains(s), "missing {s}");
        }
    }
}
