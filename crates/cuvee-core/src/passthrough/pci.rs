//! PCI devices and IOMMU groups, read from sysfs.
//!
//! Every function takes the sysfs root (`/sys` on a real host) so tests can point it at a
//! directory tree of their own.

use std::path::{Path, PathBuf};

/// One PCI function, as `/sys/bus/pci/devices/<address>` describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PciDevice {
    /// Domain, bus, slot and function: `0000:09:00.0`.
    pub address: String,
    pub vendor: u16,
    pub device: u16,
    /// Class, subclass and programming interface: `0x030000` for a VGA controller.
    pub class: u32,
    /// The driver bound to it now (`amdgpu`, `vfio-pci`), if any.
    pub driver: Option<String>,
    pub iommu_group: Option<u32>,
    /// The firmware drew its boot screen on this GPU (`boot_vga` is 1).
    pub boot_vga: bool,
}

impl PciDevice {
    /// `1002:67df`, the form `lspci -n` and `vfio-pci ids=` use.
    pub fn id(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor, self.device)
    }

    /// The address without its function: `0000:09:00`.
    pub fn slot(&self) -> &str {
        self.address
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(&self.address)
    }

    /// The function number: 0 for `0000:09:00.0`.
    pub fn function(&self) -> u8 {
        self.address
            .rsplit_once('.')
            .and_then(|(_, f)| f.parse().ok())
            .unwrap_or(0)
    }

    /// A display controller: VGA, 3D or other display class (0x03).
    pub fn is_display(&self) -> bool {
        self.class >> 16 == 0x03
    }

    /// A bridge (class 0x06): a PCIe root or switch port. Bridges may share a GPU's IOMMU group
    /// without stopping passthrough, as long as they keep their own driver.
    pub fn is_bridge(&self) -> bool {
        self.class >> 16 == 0x06
    }

    pub fn bound_to_vfio(&self) -> bool {
        self.driver.as_deref() == Some("vfio-pci")
    }
}

fn read_hex(path: &Path) -> Option<u32> {
    let s = std::fs::read_to_string(path).ok()?;
    u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
}

fn link_name(path: &Path) -> Option<String> {
    std::fs::read_link(path)
        .ok()?
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
}

fn read_device(dir: &Path) -> Option<PciDevice> {
    let address = dir.file_name()?.to_string_lossy().into_owned();
    Some(PciDevice {
        vendor: read_hex(&dir.join("vendor"))? as u16,
        device: read_hex(&dir.join("device"))? as u16,
        class: read_hex(&dir.join("class"))?,
        driver: link_name(&dir.join("driver")),
        iommu_group: link_name(&dir.join("iommu_group")).and_then(|g| g.parse().ok()),
        boot_vga: std::fs::read_to_string(dir.join("boot_vga"))
            .map(|s| s.trim() == "1")
            .unwrap_or(false),
        address,
    })
}

/// Every PCI function under `<sys>/bus/pci/devices`, sorted by address.
pub fn scan(sys: &Path) -> Vec<PciDevice> {
    let mut out: Vec<PciDevice> = std::fs::read_dir(sys.join("bus/pci/devices"))
        .map(|d| d.flatten().filter_map(|e| read_device(&e.path())).collect())
        .unwrap_or_default();
    out.sort_by(|a, b| a.address.cmp(&b.address));
    out
}

/// Whether the kernel made IOMMU groups at all (IOMMU on in the firmware and the kernel).
pub fn iommu_enabled(sys: &Path) -> bool {
    std::fs::read_dir(sys.join("kernel/iommu_groups"))
        .map(|mut d| d.next().is_some())
        .unwrap_or(false)
}

/// The device node QEMU opens for a group: `/dev/vfio/16`.
pub fn vfio_node(group: u32) -> PathBuf {
    PathBuf::from(format!("/dev/vfio/{group}"))
}

#[cfg(test)]
pub(crate) mod fake {
    //! A sysfs tree in a temporary directory, for tests.
    use std::path::{Path, PathBuf};

    pub struct Sysfs {
        pub root: PathBuf,
    }

    impl Sysfs {
        pub fn new(tag: &str) -> Sysfs {
            let root = std::env::temp_dir().join(format!(
                "cuvee-sysfs-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("bus/pci/devices")).unwrap();
            std::fs::create_dir_all(root.join("kernel/iommu_groups")).unwrap();
            std::fs::create_dir_all(root.join("bus/pci/drivers")).unwrap();
            Sysfs { root }
        }

        /// Adds a function: `class` like 0x030000, `driver` like "amdgpu".
        #[allow(clippy::too_many_arguments)]
        pub fn add(
            &self,
            address: &str,
            vendor: u16,
            device: u16,
            class: u32,
            driver: Option<&str>,
            group: u32,
            boot_vga: bool,
        ) -> &Self {
            let dir = self.root.join("bus/pci/devices").join(address);
            std::fs::create_dir_all(&dir).unwrap();
            let w = |n: &str, v: String| std::fs::write(dir.join(n), v + "\n").unwrap();
            w("vendor", format!("0x{vendor:04x}"));
            w("device", format!("0x{device:04x}"));
            w("class", format!("0x{class:06x}"));
            if class >> 16 == 0x03 {
                w("boot_vga", if boot_vga { "1" } else { "0" }.into());
            }
            let g = self
                .root
                .join("kernel/iommu_groups")
                .join(group.to_string());
            std::fs::create_dir_all(g.join("devices")).unwrap();
            let _ = std::os::unix::fs::symlink(&dir, g.join("devices").join(address));
            std::os::unix::fs::symlink(&g, dir.join("iommu_group")).unwrap();
            if let Some(d) = driver {
                let dd = self.root.join("bus/pci/drivers").join(d);
                std::fs::create_dir_all(&dd).unwrap();
                std::os::unix::fs::symlink(&dd, dir.join("driver")).unwrap();
            }
            self
        }

        pub fn path(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for Sysfs {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_devices_groups_and_drivers() {
        let s = fake::Sysfs::new("pci");
        s.add(
            "0000:09:00.0",
            0x10de,
            0x2f04,
            0x030000,
            Some("nvidia"),
            16,
            true,
        )
        .add(
            "0000:09:00.1",
            0x10de,
            0x2f80,
            0x040300,
            Some("snd_hda_intel"),
            16,
            false,
        );
        let devs = scan(s.path());
        assert_eq!(devs.len(), 2);
        let gpu = &devs[0];
        assert_eq!(gpu.id(), "10de:2f04");
        assert_eq!(gpu.slot(), "0000:09:00");
        assert_eq!(gpu.function(), 0);
        assert_eq!(gpu.driver.as_deref(), Some("nvidia"));
        assert_eq!(gpu.iommu_group, Some(16));
        assert!(gpu.is_display() && gpu.boot_vga);
        assert!(!devs[1].is_display());
        assert_eq!(devs[1].function(), 1);
        assert!(iommu_enabled(s.path()));
    }

    #[test]
    fn no_groups_means_iommu_off() {
        let s = fake::Sysfs::new("noiommu");
        assert!(!iommu_enabled(s.path()));
        assert!(scan(s.path()).is_empty());
    }
}
