//! The host's distribution and its package manager: which packages the runtime needs there,
//! which are missing, and the command that installs them.
//!
//! Four families, by `/etc/os-release`'s `ID` and `ID_LIKE`: Debian and Ubuntu (apt), Fedora
//! and its relatives (dnf), Arch and its derivatives (pacman), openSUSE (zypper). Any other
//! gets the Ubuntu names to look up by hand.

use crate::host::GpuVendor;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Apt,
    Dnf,
    Pacman,
    Zypper,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distro {
    pub id: String,
    pub like: Vec<String>,
    /// `PRETTY_NAME`, for messages.
    pub name: String,
    pub family: Family,
}

impl Distro {
    pub fn detect() -> Distro {
        let text = std::fs::read_to_string("/etc/os-release")
            .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
            .unwrap_or_default();
        Self::parse(&text)
    }

    pub fn parse(os_release: &str) -> Distro {
        let field = |k: &str| {
            os_release
                .lines()
                .find_map(|l| l.strip_prefix(k)?.strip_prefix('='))
                .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .unwrap_or_default()
        };
        let id = field("ID").to_ascii_lowercase();
        let like: Vec<String> = field("ID_LIKE")
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        let mut name = field("PRETTY_NAME");
        if name.is_empty() {
            name = if id.is_empty() {
                "Linux".into()
            } else {
                id.clone()
            };
        }
        let is = |x: &str| id == x || like.iter().any(|l| l == x);
        let family = if is("debian") || is("ubuntu") {
            Family::Apt
        } else if is("fedora") || is("rhel") || is("centos") {
            Family::Dnf
        } else if is("arch") || is("archlinux") {
            Family::Pacman
        } else if is("suse") || is("opensuse") || id.starts_with("opensuse") {
            Family::Zypper
        } else {
            Family::Unknown
        };
        Distro {
            id,
            like,
            name,
            family,
        }
    }
}

/// A package the runtime needs, by family. `a|b` is a choice: the first one the package
/// manager knows (Ubuntu 24.04 renamed some libraries with a `t64` suffix).
struct Need {
    apt: &'static str,
    dnf: &'static str,
    pacman: &'static str,
    zypper: &'static str,
}

/// The libraries QEMU loads (packaging/runtime/build.sh writes them to NEEDED.txt), curl
/// for the downloads and the Vulkan loader.
const RUNTIME: &[Need] = &[
    Need {
        apt: "libglib2.0-0t64|libglib2.0-0",
        dnf: "glib2",
        pacman: "glib2",
        zypper: "libglib-2_0-0",
    },
    Need {
        apt: "libpixman-1-0",
        dnf: "pixman",
        pacman: "pixman",
        zypper: "libpixman-1-0",
    },
    Need {
        apt: "libgtk-3-0t64|libgtk-3-0",
        dnf: "gtk3",
        pacman: "gtk3",
        zypper: "libgtk-3-0",
    },
    Need {
        apt: "libslirp0",
        dnf: "libslirp",
        pacman: "libslirp",
        zypper: "libslirp0",
    },
    Need {
        apt: "libusb-1.0-0",
        dnf: "libusb1",
        pacman: "libusb",
        zypper: "libusb-1_0-0",
    },
    Need {
        apt: "libpulse0",
        dnf: "pulseaudio-libs",
        pacman: "libpulse",
        zypper: "libpulse0",
    },
    Need {
        apt: "libasound2t64|libasound2",
        dnf: "alsa-lib",
        pacman: "alsa-lib",
        zypper: "libasound2",
    },
    Need {
        apt: "libzstd1",
        dnf: "libzstd",
        pacman: "zstd",
        zypper: "libzstd1",
    },
    Need {
        apt: "zlib1g",
        dnf: "zlib-ng-compat|zlib",
        pacman: "zlib",
        zypper: "libz1",
    },
    Need {
        apt: "libvulkan1",
        dnf: "vulkan-loader",
        pacman: "vulkan-icd-loader",
        zypper: "libvulkan1",
    },
    Need {
        apt: "vulkan-tools",
        dnf: "vulkan-tools",
        pacman: "vulkan-tools",
        zypper: "vulkan-tools",
    },
    Need {
        apt: "curl",
        dnf: "curl",
        pacman: "curl",
        zypper: "curl",
    },
    Need {
        apt: "ca-certificates",
        dnf: "ca-certificates",
        pacman: "ca-certificates",
        zypper: "ca-certificates",
    },
    Need {
        apt: "kmod",
        dnf: "kmod",
        pacman: "kmod",
        zypper: "kmod",
    },
];

/// Mesa's Vulkan driver for AMD and Intel. NVIDIA's comes with its own driver, which the
/// distribution installs its own way: it is checked, not installed.
const MESA_AMD: Need = Need {
    apt: "mesa-vulkan-drivers",
    dnf: "mesa-vulkan-drivers",
    pacman: "vulkan-radeon",
    zypper: "libvulkan_radeon",
};
const MESA_INTEL: Need = Need {
    apt: "mesa-vulkan-drivers",
    dnf: "mesa-vulkan-drivers",
    pacman: "vulkan-intel",
    zypper: "libvulkan_intel",
};

impl Need {
    fn get(&self, f: Family) -> &'static str {
        match f {
            Family::Apt | Family::Unknown => self.apt,
            Family::Dnf => self.dnf,
            Family::Pacman => self.pacman,
            Family::Zypper => self.zypper,
        }
    }
}

/// The packages to have, each a list of choices, for `family` and the host's GPUs.
pub fn wanted(family: Family, gpus: &[GpuVendor]) -> Vec<Vec<&'static str>> {
    let mut out: Vec<Vec<&'static str>> = RUNTIME
        .iter()
        .map(|n| n.get(family).split('|').collect())
        .collect();
    for (vendor, need) in [(GpuVendor::Amd, &MESA_AMD), (GpuVendor::Intel, &MESA_INTEL)] {
        if gpus.contains(&vendor) {
            let p: Vec<&str> = need.get(family).split('|').collect();
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

fn quiet(cmd: &mut Command) -> bool {
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Whether `pkg` is installed.
pub fn installed(family: Family, pkg: &str) -> bool {
    match family {
        Family::Apt => Command::new("dpkg-query")
            .args(["-W", "-f=${Status}", pkg])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("install ok installed"))
            .unwrap_or(false),
        Family::Dnf | Family::Zypper => {
            quiet(Command::new("rpm").args(["-q", "--whatprovides", pkg]))
        }
        Family::Pacman => quiet(Command::new("pacman").args(["-Q", pkg])),
        Family::Unknown => false,
    }
}

/// Whether the package manager can install `pkg` (its lists name it).
pub fn available(family: Family, pkg: &str) -> bool {
    match family {
        Family::Apt => Command::new("apt-cache")
            .args(["show", "--no-all-versions", pkg])
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false),
        Family::Dnf => quiet(Command::new("dnf").args(["-q", "info", pkg])),
        Family::Pacman => quiet(Command::new("pacman").args(["-Si", pkg])),
        Family::Zypper => Command::new("zypper")
            .args(["--non-interactive", "--quiet", "info", pkg])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Name"))
            .unwrap_or(false),
        Family::Unknown => false,
    }
}

/// The choices none of whose packages are installed, resolved to the package to install:
/// the first choice the package manager knows, else the first.
pub fn missing(family: Family, wanted: &[Vec<&'static str>]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for choices in wanted {
        if choices.iter().any(|p| installed(family, p)) {
            continue;
        }
        let pick = if choices.len() == 1 {
            choices[0]
        } else {
            choices
                .iter()
                .copied()
                .find(|p| available(family, p))
                .unwrap_or(choices[0])
        };
        if !out.contains(&pick) {
            out.push(pick);
        }
    }
    out
}

/// The commands that refresh the package lists (apt only) and install `pkgs`, without
/// `sudo` in front. `yes` answers the package manager's questions.
pub fn install_commands(family: Family, pkgs: &[&str], yes: bool) -> Vec<Vec<String>> {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    let mut cmds = Vec::new();
    let mut install = match family {
        Family::Apt => {
            cmds.push(s(&["apt-get", "update"]));
            let mut c = s(&["apt-get", "install", "--no-install-recommends"]);
            if yes {
                c.push("-y".into());
            }
            c
        }
        Family::Dnf => {
            let mut c = s(&["dnf", "install"]);
            if yes {
                c.push("-y".into());
            }
            c
        }
        Family::Pacman => {
            let mut c = s(&["pacman", "-S", "--needed"]);
            if yes {
                c.push("--noconfirm".into());
            }
            c
        }
        Family::Zypper => {
            let mut c = s(&["zypper"]);
            if yes {
                c.push("--non-interactive".into());
            }
            c.push("install".into());
            c
        }
        Family::Unknown => return Vec::new(),
    };
    install.extend(pkgs.iter().map(|p| p.to_string()));
    cmds.push(install);
    cmds
}

/// Whether NVIDIA's Vulkan driver is installed (its ICD file).
pub fn nvidia_vulkan_installed() -> bool {
    ["/usr/share/vulkan/icd.d", "/etc/vulkan/icd.d"]
        .iter()
        .any(|d| {
            std::fs::read_dir(d)
                .map(|rd| {
                    rd.flatten()
                        .any(|e| e.file_name().to_string_lossy().starts_with("nvidia"))
                })
                .unwrap_or(false)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_from_os_release() {
        let f = |s: &str| Distro::parse(s).family;
        assert_eq!(f("ID=ubuntu\nID_LIKE=debian\n"), Family::Apt);
        assert_eq!(f("ID=debian\n"), Family::Apt);
        assert_eq!(f("ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n"), Family::Apt);
        assert_eq!(f("ID=fedora\n"), Family::Dnf);
        assert_eq!(
            f("ID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\n"),
            Family::Dnf
        );
        assert_eq!(f("ID=arch\n"), Family::Pacman);
        assert_eq!(f("ID=cachyos\nID_LIKE=arch\n"), Family::Pacman);
        assert_eq!(f("ID=endeavouros\nID_LIKE=arch\n"), Family::Pacman);
        assert_eq!(
            f("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n"),
            Family::Zypper
        );
        assert_eq!(f("ID=opensuse-leap\n"), Family::Zypper);
        assert_eq!(f("ID=nixos\n"), Family::Unknown);
        assert_eq!(f(""), Family::Unknown);
    }

    #[test]
    fn names_come_from_pretty_name() {
        let d = Distro::parse("NAME=Fedora\nPRETTY_NAME=\"Fedora Linux 43\"\nID=fedora\n");
        assert_eq!(d.name, "Fedora Linux 43");
        assert_eq!(Distro::parse("ID=foo").name, "foo");
    }

    #[test]
    fn mesa_only_for_amd_and_intel() {
        let none = wanted(Family::Pacman, &[GpuVendor::Nvidia]);
        assert!(!none
            .iter()
            .flatten()
            .any(|p| p.starts_with("vulkan-radeon")));
        let both = wanted(Family::Apt, &[GpuVendor::Amd, GpuVendor::Intel]);
        let mesa = both
            .iter()
            .filter(|c| c[0] == "mesa-vulkan-drivers")
            .count();
        assert_eq!(mesa, 1);
        let p = wanted(Family::Pacman, &[GpuVendor::Amd, GpuVendor::Intel]);
        assert!(p.contains(&vec!["vulkan-radeon"]) && p.contains(&vec!["vulkan-intel"]));
    }

    #[test]
    fn install_commands_per_family() {
        let c = install_commands(Family::Apt, &["a"], true);
        assert_eq!(c[0], ["apt-get", "update"]);
        assert_eq!(
            c[1],
            ["apt-get", "install", "--no-install-recommends", "-y", "a"]
        );
        let c = install_commands(Family::Zypper, &["a"], true);
        assert_eq!(c[0], ["zypper", "--non-interactive", "install", "a"]);
        let c = install_commands(Family::Pacman, &["a"], false);
        assert_eq!(c[0], ["pacman", "-S", "--needed", "a"]);
        assert!(install_commands(Family::Unknown, &["a"], true).is_empty());
    }
}
