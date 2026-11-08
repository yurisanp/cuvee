//! The host checks: the same ones `cuvee doctor` prints, as data the GUI can draw.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuVendor {
    Amd,
    Intel,
    Nvidia,
    Other,
}

impl GpuVendor {
    fn from_pci(id: &str) -> Self {
        match id.trim() {
            "0x1002" => Self::Amd,
            "0x8086" => Self::Intel,
            "0x10de" => Self::Nvidia,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Wayland,
    X11,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Bad,
}

/// Which check: the GUI maps these to translated titles and advice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckId {
    CpuVirt,
    Kvm,
    ReimsTools,
    Vulkan,
    Udmabuf,
    MaxMapCount,
    Session,
    Disk,
    Iommu,
}

impl CheckId {
    pub const ALL: [CheckId; 9] = [
        CheckId::CpuVirt,
        CheckId::Kvm,
        CheckId::ReimsTools,
        CheckId::Vulkan,
        CheckId::Udmabuf,
        CheckId::MaxMapCount,
        CheckId::Session,
        CheckId::Disk,
        CheckId::Iommu,
    ];

    /// The Fluent message id of this check's title.
    pub fn title_key(self) -> &'static str {
        match self {
            CheckId::CpuVirt => "check-cpu-virt",
            CheckId::Kvm => "check-kvm",
            CheckId::ReimsTools => "check-reims-tools",
            CheckId::Vulkan => "check-vulkan",
            CheckId::Udmabuf => "check-udmabuf",
            CheckId::MaxMapCount => "check-max-map-count",
            CheckId::Session => "check-session",
            CheckId::Disk => "check-disk",
            CheckId::Iommu => "check-iommu",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub id: CheckId,
    pub level: Level,
    /// The Fluent id of the explanation shown when the check is not Ok.
    pub advice_key: Option<&'static str>,
    /// The command that fixes it, if one does.
    pub fix_command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub checks: Vec<Check>,
    pub gpus: Vec<GpuVendor>,
    pub session: Session,
    pub reims_ctl: Option<PathBuf>,
    pub reims_util: Option<PathBuf>,
    pub qemu: Option<PathBuf>,
}

impl Report {
    pub fn ok_count(&self) -> usize {
        self.checks.iter().filter(|c| c.level == Level::Ok).count()
    }

    pub fn worst(&self) -> Level {
        if self.checks.iter().any(|c| c.level == Level::Bad) {
            Level::Bad
        } else if self.checks.iter().any(|c| c.level == Level::Warn) {
            Level::Warn
        } else {
            Level::Ok
        }
    }

    /// The first check that needs attention, worst first.
    pub fn first_problem(&self) -> Option<&Check> {
        self.checks
            .iter()
            .find(|c| c.level == Level::Bad)
            .or_else(|| self.checks.iter().find(|c| c.level == Level::Warn))
    }
}

fn readable_writable(p: &str) -> bool {
    let path = Path::new(p);
    path.exists()
        && std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .is_ok()
}

pub fn gpu_vendors() -> Vec<GpuVendor> {
    let mut out = Vec::new();
    if let Ok(cards) = std::fs::read_dir("/sys/class/drm") {
        for card in cards.flatten() {
            let name = card.file_name();
            let name = name.to_string_lossy();
            // card0, card1: not card0-HDMI-A-1.
            if !name.starts_with("card") || name.contains('-') {
                continue;
            }
            if let Ok(id) = std::fs::read_to_string(card.path().join("device/vendor")) {
                let v = GpuVendor::from_pci(&id);
                if !out.contains(&v) {
                    out.push(v);
                }
            }
        }
    }
    out
}

pub fn session() -> Session {
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => Session::Wayland,
        Ok("x11") => Session::X11,
        _ => {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                Session::Wayland
            } else if std::env::var_os("DISPLAY").is_some() {
                Session::X11
            } else {
                Session::None
            }
        }
    }
}

use crate::paths::which;

fn cpu_has_virt() -> bool {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|s| {
            s.lines()
                .filter(|l| l.starts_with("flags"))
                .any(|l| l.split_whitespace().any(|f| f == "vmx" || f == "svm"))
        })
        .unwrap_or(false)
}

fn vulkan_works() -> Option<bool> {
    let bin = which("vulkaninfo")?;
    Some(
        Command::new(bin)
            .arg("--summary")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
    )
}

fn max_map_count() -> Option<u64> {
    std::fs::read_to_string("/proc/sys/vm/max_map_count")
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// The vm.max_map_count a new VM needs on NVIDIA so the GPU maps its memory without copies,
/// as reims-ctl and upstream's install script work it out: a new VM gets half the host's
/// memory (4 to 16 GiB), a mapping per 4 KiB page plus 24,576 spare, rounded up to a power
/// of two from 1,048,576.
pub fn wanted_max_map_count() -> u64 {
    let mem_kib = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(0);
    max_map_count_for(mem_kib)
}

fn max_map_count_for(mem_kib: u64) -> u64 {
    let vm_mib = (mem_kib / 2 / 1_048_576 * 1024).clamp(4096, 16384);
    let want = vm_mib * 256 + 24_576;
    let mut raise = 1_048_576;
    while raise < want {
        raise *= 2;
    }
    raise
}

/// Free bytes on the file system that holds the user's data directory.
fn free_bytes(path: &Path) -> Option<u64> {
    // `df` keeps this free of libc bindings; the number is only advisory.
    let out = Command::new("df")
        .args(["--output=avail", "-B1"])
        .arg(path)
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .nth(1)?
        .trim()
        .parse()
        .ok()
}

fn data_dir() -> PathBuf {
    if let Some(v) = std::env::var_os("XDG_DATA_HOME").filter(|v| Path::new(v).is_absolute()) {
        return PathBuf::from(v);
    }
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
}

/// Runs every check. Nothing here changes the system.
pub fn inspect() -> Report {
    let gpus = gpu_vendors();
    let session = session();
    let reims_ctl = crate::paths::reims_ctl();
    let reims_util = crate::paths::reims_util();
    let qemu = crate::paths::qemu_binary().filter(|q| q.is_file());
    let mut checks = Vec::with_capacity(CheckId::ALL.len());

    checks.push(if cpu_has_virt() {
        Check {
            id: CheckId::CpuVirt,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        }
    } else {
        Check {
            id: CheckId::CpuVirt,
            level: Level::Bad,
            advice_key: Some("advice-cpu-virt"),
            fix_command: None,
        }
    });

    checks.push(if !Path::new("/dev/kvm").exists() {
        Check {
            id: CheckId::Kvm,
            level: Level::Bad,
            advice_key: Some("advice-kvm-missing"),
            fix_command: Some("sudo modprobe kvm_intel || sudo modprobe kvm_amd".into()),
        }
    } else if readable_writable("/dev/kvm") {
        Check {
            id: CheckId::Kvm,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        }
    } else {
        Check {
            id: CheckId::Kvm,
            level: Level::Bad,
            advice_key: Some("advice-kvm-access"),
            fix_command: Some("cuvee setup-host".into()),
        }
    });

    checks.push(
        if reims_ctl.is_some() && reims_util.is_some() && qemu.is_some() {
            Check {
                id: CheckId::ReimsTools,
                level: Level::Ok,
                advice_key: None,
                fix_command: None,
            }
        } else {
            Check {
                id: CheckId::ReimsTools,
                level: Level::Bad,
                advice_key: Some("advice-reims-tools"),
                fix_command: crate::paths::packaged()
                    .is_none()
                    .then(|| "cuvee install".into()),
            }
        },
    );

    checks.push(match vulkan_works() {
        Some(true) => Check {
            id: CheckId::Vulkan,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        },
        Some(false) => Check {
            id: CheckId::Vulkan,
            level: Level::Bad,
            advice_key: Some("advice-vulkan-broken"),
            fix_command: None,
        },
        None => Check {
            id: CheckId::Vulkan,
            level: Level::Warn,
            advice_key: Some("advice-vulkan-untested"),
            fix_command: None,
        },
    });

    checks.push(if !Path::new("/dev/udmabuf").exists() {
        Check {
            id: CheckId::Udmabuf,
            level: Level::Bad,
            advice_key: Some("advice-udmabuf-missing"),
            fix_command: Some("sudo modprobe udmabuf".into()),
        }
    } else if readable_writable("/dev/udmabuf") {
        Check {
            id: CheckId::Udmabuf,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        }
    } else {
        Check {
            id: CheckId::Udmabuf,
            level: Level::Bad,
            advice_key: Some("advice-udmabuf-access"),
            fix_command: Some("cuvee setup-host".into()),
        }
    });

    // Only NVIDIA pays for a low vm.max_map_count (reims-vgpu docs).
    let nvidia = gpus.contains(&GpuVendor::Nvidia);
    let want = wanted_max_map_count();
    checks.push(match max_map_count() {
        Some(n) if nvidia && n < want => Check {
            id: CheckId::MaxMapCount,
            level: Level::Warn,
            advice_key: Some("advice-max-map-count"),
            fix_command: Some("cuvee setup-host".into()),
        },
        _ => Check {
            id: CheckId::MaxMapCount,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        },
    });

    checks.push(match session {
        Session::None => Check {
            id: CheckId::Session,
            level: Level::Warn,
            advice_key: Some("advice-session-none"),
            fix_command: None,
        },
        _ => Check {
            id: CheckId::Session,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        },
    });

    // A macOS installer plus a system disk needs about 64 GiB to be comfortable.
    checks.push(match free_bytes(&data_dir()) {
        Some(b) if b < 64 * 1024 * 1024 * 1024 => Check {
            id: CheckId::Disk,
            level: Level::Warn,
            advice_key: Some("advice-disk-low"),
            fix_command: None,
        },
        _ => Check {
            id: CheckId::Disk,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        },
    });

    let iommu = std::fs::read_dir("/sys/kernel/iommu_groups")
        .map(|d| d.count() > 0)
        .unwrap_or(false);
    checks.push(if iommu {
        Check {
            id: CheckId::Iommu,
            level: Level::Ok,
            advice_key: None,
            fix_command: None,
        }
    } else {
        Check {
            id: CheckId::Iommu,
            level: Level::Warn,
            advice_key: Some("advice-iommu-off"),
            fix_command: None,
        }
    });

    Report {
        checks,
        gpus,
        session,
        reims_ctl,
        reims_util,
        qemu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_check_is_reported_once() {
        let r = inspect();
        assert_eq!(r.checks.len(), CheckId::ALL.len());
        for id in CheckId::ALL {
            assert_eq!(r.checks.iter().filter(|c| c.id == id).count(), 1);
        }
    }

    #[test]
    fn max_map_count_follows_memory() {
        assert_eq!(max_map_count_for(8 * 1024 * 1024), 2_097_152);
        assert_eq!(max_map_count_for(32 * 1024 * 1024), 8_388_608);
        assert_eq!(max_map_count_for(128 * 1024 * 1024), 8_388_608);
        assert_eq!(max_map_count_for(0), 2_097_152);
    }

    #[test]
    fn pci_ids_map_to_vendors() {
        assert_eq!(GpuVendor::from_pci("0x1002\n"), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_pci("0x10de"), GpuVendor::Nvidia);
        assert_eq!(GpuVendor::from_pci("0x8086"), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_pci("0x1234"), GpuVendor::Other);
    }
}
