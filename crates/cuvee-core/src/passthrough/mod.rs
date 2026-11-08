//! GPU passthrough (VFIO): giving the VM a physical GPU, so macOS 10.13 to 15 run with Metal.
//!
//! reims-vgpu accelerates macOS 26 alone. Older guests get acceleration only from a real GPU
//! that macOS has a driver for, handed to the VM whole through the kernel's `vfio-pci`. Cuvée
//! does not run QEMU itself: it reads the host ([`pci`]), says whether a GPU fits a macOS
//! version ([`catalog`], [`assess`]), writes the host's settings ([`host_plan`]), and gives
//! reims-ctl the QEMU devices through `reims-ctl set --extra-args` ([`attach_command`]).
//!
//! For GPUs macOS dropped (Kepler, AMD GCN 1–3 on 13 to 15), the guest needs OpenCore Legacy
//! Patcher's root patches. Cuvée does not ship OCLP: [`oclp`] only sets up the VM's OpenCore
//! the way OCLP's root patches need (SIP and AMFI), and the person downloads OCLP from its own
//! releases inside macOS.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

pub mod catalog;
pub mod oclp;
pub mod pci;

use crate::macos::Version;
use catalog::{Family, Support};
use pci::PciDevice;
use std::path::{Path, PathBuf};

pub const SYSFS: &str = "/sys";

/// A GPU the VM could get: its display function and the functions that go with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gpu {
    /// The display function (usually `.0`).
    pub primary: PciDevice,
    /// Every function on the card's slot, the display function first: its HDMI audio, USB-C
    /// controller and so on. They all go to the VM together.
    pub functions: Vec<PciDevice>,
    /// Devices in the same IOMMU group that are not the card's own and not bridges. VFIO hands
    /// a VM a whole group, so these would go too: passthrough needs this empty.
    pub group_others: Vec<PciDevice>,
    pub family: Family,
}

impl Gpu {
    /// The `vendor:device` IDs of the card's functions, for `vfio-pci ids=`.
    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.functions.iter().map(PciDevice::id).collect();
        ids.dedup();
        ids
    }

    pub fn group(&self) -> Option<u32> {
        self.primary.iommu_group
    }
}

/// The GPUs on the host, from a scan of its PCI devices.
pub fn gpus(devices: &[PciDevice]) -> Vec<Gpu> {
    devices
        .iter()
        .filter(|d| d.is_display() && d.function() == 0)
        .map(|primary| {
            let mut functions: Vec<PciDevice> = devices
                .iter()
                .filter(|d| d.slot() == primary.slot())
                .cloned()
                .collect();
            functions.sort_by_key(PciDevice::function);
            let group_others = match primary.iommu_group {
                Some(g) => devices
                    .iter()
                    .filter(|d| {
                        d.iommu_group == Some(g) && d.slot() != primary.slot() && !d.is_bridge()
                    })
                    .cloned()
                    .collect(),
                None => Vec::new(),
            };
            Gpu {
                family: Family::of(primary.vendor, primary.device),
                primary: primary.clone(),
                functions,
                group_others,
            }
        })
        .collect()
}

/// Finds a GPU by its address, with or without the domain and function: `09:00`, `0000:09:00.0`.
pub fn find<'a>(gpus: &'a [Gpu], address: &str) -> Option<&'a Gpu> {
    let mut want = address.trim().to_ascii_lowercase();
    if want.matches(':').count() == 1 {
        want = format!("0000:{want}");
    }
    gpus.iter().find(|g| {
        if want.contains('.') {
            g.primary.address == want
        } else {
            g.primary.slot() == want
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Other,
}

/// What [`assess`] needs from the host besides the GPU itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostState {
    pub iommu: bool,
    pub cpu: CpuVendor,
    /// The kernel command line: `/proc/cmdline`.
    pub cmdline: String,
    /// The soft limit on locked memory of this process, None for unlimited. QEMU locks all of
    /// the VM's RAM when it has a VFIO device.
    pub memlock_bytes: Option<u64>,
    /// How many display controllers the host has.
    pub displays: usize,
}

pub fn cpu_vendor() -> CpuVendor {
    let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    match info
        .lines()
        .find(|l| l.starts_with("vendor_id"))
        .and_then(|l| l.split(':').nth(1))
        .map(str::trim)
    {
        Some("GenuineIntel") => CpuVendor::Intel,
        Some("AuthenticAMD") => CpuVendor::Amd,
        _ => CpuVendor::Other,
    }
}

/// Parses `/proc/<pid>/limits`' "Max locked memory" soft limit; None for unlimited.
pub fn parse_memlock(limits: &str) -> Option<Option<u64>> {
    let line = limits
        .lines()
        .find(|l| l.starts_with("Max locked memory"))?;
    let soft = line
        .trim_start_matches("Max locked memory")
        .split_whitespace()
        .next()?;
    if soft == "unlimited" {
        Some(None)
    } else {
        soft.parse().ok().map(Some)
    }
}

pub fn host_state(sys: &Path, devices: &[PciDevice]) -> HostState {
    HostState {
        iommu: pci::iommu_enabled(sys),
        cpu: cpu_vendor(),
        cmdline: std::fs::read_to_string("/proc/cmdline").unwrap_or_default(),
        memlock_bytes: std::fs::read_to_string("/proc/self/limits")
            .ok()
            .and_then(|l| parse_memlock(&l))
            .unwrap_or(Some(0)),
        displays: devices.iter().filter(|d| d.is_display()).count(),
    }
}

/// The kernel parameters passthrough wants on this CPU. AMD-Vi is on by default; Intel's VT-d
/// needs `intel_iommu=on`. `iommu=pt` keeps the host's own devices fast.
pub fn kernel_params(cpu: CpuVendor) -> &'static [&'static str] {
    match cpu {
        CpuVendor::Intel => &["intel_iommu=on", "iommu=pt"],
        _ => &["iommu=pt"],
    }
}

/// The parameters from [`kernel_params`] that `cmdline` lacks.
pub fn missing_kernel_params(cpu: CpuVendor, cmdline: &str) -> Vec<&'static str> {
    let have: Vec<&str> = cmdline.split_whitespace().collect();
    kernel_params(cpu)
        .iter()
        .copied()
        .filter(|p| !have.contains(p))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Ok,
    Warn,
    Bad,
}

/// One thing [`assess`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// No IOMMU groups: IOMMU is off in the firmware or the kernel.
    IommuOff { missing_params: Vec<&'static str> },
    /// macOS has no driver for this GPU on this version.
    Unsupported(Support),
    /// macOS dropped the driver; OCLP's root patches are needed inside the guest.
    NeedsOclp,
    /// Works, with a catch (Fluent id).
    Caveat(&'static str),
    /// The GPU stays stuck after the VM shuts down until the host reboots.
    ResetBug,
    /// Other devices share the GPU's IOMMU group.
    GroupShared(Vec<String>),
    /// The host draws its own screen on this GPU; `only` when it has no other.
    HostScreen { only: bool },
    /// The card is not bound to vfio-pci yet (its current drivers).
    NotBound(Vec<String>),
    /// The group's /dev/vfio node is not open to this user.
    VfioAccess(u32),
    /// The locked-memory limit is below the VM's RAM.
    Memlock { have: u64, need: u64 },
}

impl Finding {
    pub fn level(&self) -> Level {
        match self {
            Finding::IommuOff { .. }
            | Finding::Unsupported(_)
            | Finding::GroupShared(_)
            | Finding::HostScreen { only: true } => Level::Bad,
            Finding::NeedsOclp
            | Finding::Caveat(_)
            | Finding::ResetBug
            | Finding::HostScreen { only: false }
            | Finding::NotBound(_)
            | Finding::VfioAccess(_)
            | Finding::Memlock { .. } => Level::Warn,
        }
    }

    /// The Fluent id of the explanation.
    pub fn key(&self) -> &'static str {
        match self {
            Finding::IommuOff { .. } => "pt-find-iommu-off",
            Finding::Unsupported(Support::WebDriver) => "pt-find-webdriver",
            Finding::Unsupported(_) => "pt-find-unsupported",
            Finding::NeedsOclp => "pt-find-oclp",
            Finding::Caveat(k) => k,
            Finding::ResetBug => "pt-find-reset-bug",
            Finding::GroupShared(_) => "pt-find-group-shared",
            Finding::HostScreen { only: true } => "pt-find-host-screen-only",
            Finding::HostScreen { only: false } => "pt-find-host-screen",
            Finding::NotBound(_) => "pt-find-not-bound",
            Finding::VfioAccess(_) => "pt-find-vfio-access",
            Finding::Memlock { .. } => "pt-find-memlock",
        }
    }

    /// Whether `cuvee passthrough setup-host` fixes it.
    pub fn fixed_by_setup(&self) -> bool {
        matches!(
            self,
            Finding::NotBound(_) | Finding::VfioAccess(_) | Finding::Memlock { .. }
        )
    }
}

/// Everything [`assess`] found, worst first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub support: Support,
    pub findings: Vec<Finding>,
}

impl Assessment {
    pub fn worst(&self) -> Level {
        self.findings
            .iter()
            .map(Finding::level)
            .max()
            .unwrap_or(Level::Ok)
    }

    /// No finding stops the VM from starting with this GPU.
    pub fn ready(&self) -> bool {
        self.worst() < Level::Bad && !self.findings.iter().any(Finding::fixed_by_setup)
    }

    /// Possible once `setup-host` has run.
    pub fn possible(&self) -> bool {
        self.worst() < Level::Bad
    }
}

/// RAM QEMU locks besides the guest's: its own buffers and the GPU's BARs, with room to spare.
const MEMLOCK_MARGIN: u64 = 1 << 30;

/// Whether `gpu` can be passed to a VM of `version` with `memory_gib` of RAM, on `host`.
pub fn assess(gpu: &Gpu, version: Version, memory_gib: u32, host: &HostState) -> Assessment {
    let support = gpu.family.support(version);
    let mut f = Vec::new();
    if !host.iommu {
        f.push(Finding::IommuOff {
            missing_params: missing_kernel_params(host.cpu, &host.cmdline),
        });
    }
    match support {
        Support::Native => {}
        Support::NativeCaveat(k) => f.push(Finding::Caveat(k)),
        Support::OclpRootPatch => f.push(Finding::NeedsOclp),
        s @ (Support::WebDriver | Support::Unsupported) => f.push(Finding::Unsupported(s)),
    }
    if gpu.family.reset_bug() {
        f.push(Finding::ResetBug);
    }
    if !gpu.group_others.is_empty() {
        f.push(Finding::GroupShared(
            gpu.group_others.iter().map(|d| d.address.clone()).collect(),
        ));
    }
    if gpu.primary.boot_vga || host.displays <= 1 {
        f.push(Finding::HostScreen {
            only: host.displays <= 1,
        });
    }
    let unbound: Vec<String> = gpu
        .functions
        .iter()
        .filter(|d| !d.bound_to_vfio())
        .map(|d| format!("{} ({})", d.address, d.driver.as_deref().unwrap_or("-")))
        .collect();
    if !unbound.is_empty() {
        f.push(Finding::NotBound(unbound));
    } else if let Some(g) = gpu.group() {
        let node = pci::vfio_node(g);
        let open = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&node)
            .is_ok();
        if !open {
            f.push(Finding::VfioAccess(g));
        }
    }
    let need = u64::from(memory_gib) * (1 << 30) + MEMLOCK_MARGIN;
    if let Some(have) = host.memlock_bytes {
        if have < need {
            f.push(Finding::Memlock { have, need });
        }
    }
    f.sort_by_key(|x| std::cmp::Reverse(x.level()));
    Assessment {
        support,
        findings: f,
    }
}

/// The host's initramfs generator: vfio-pci must load before the GPU's own driver, so it goes
/// into the initramfs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Initramfs {
    Mkinitcpio,
    Dracut,
    InitramfsTools,
    Unknown,
}

fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
        || Path::new("/usr/sbin").join(bin).is_file()
}

pub fn initramfs() -> Initramfs {
    if on_path("mkinitcpio") {
        Initramfs::Mkinitcpio
    } else if on_path("dracut") {
        Initramfs::Dracut
    } else if on_path("update-initramfs") {
        Initramfs::InitramfsTools
    } else {
        Initramfs::Unknown
    }
}

/// A file [`host_plan`] writes, whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub contents: String,
}

/// What setting the host up for one GPU takes. Everything here needs root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostPlan {
    /// Files to write: they make the setup survive reboots.
    pub files: Vec<FileChange>,
    /// Commands to run after the files: reload udev, rebuild the initramfs.
    pub commands: Vec<String>,
    /// Commands that bind the card to vfio-pci now, without a reboot. They fail while the host
    /// draws on the card.
    pub bind_now: Vec<String>,
    /// Kernel parameters to add to the boot loader by hand (Cuvée does not edit boot loaders).
    pub kernel_params: Vec<&'static str>,
}

pub const MODPROBE_FILE: &str = "/etc/modprobe.d/cuvee-vfio.conf";
pub const UDEV_FILE: &str = "/etc/udev/rules.d/71-cuvee-vfio.rules";
pub const LIMITS_FILE: &str = "/etc/security/limits.d/90-cuvee-memlock.conf";
pub const USER_SERVICE_FILE: &str = "/etc/systemd/system/user@.service.d/90-cuvee-memlock.conf";

/// The drivers that could claim a passthrough card's functions before vfio-pci does.
const HOST_DRIVERS: [&str; 7] = [
    "amdgpu",
    "radeon",
    "nouveau",
    "nvidia",
    "nvidiafb",
    "snd_hda_intel",
    "xhci_pci",
];

pub fn host_plan(gpu: &Gpu, init: Initramfs, cpu: CpuVendor, cmdline: &str) -> HostPlan {
    let mut modprobe = format!(
        "# Written by Cuvée (cuvee passthrough setup-host) for the GPU at {}.\n\
         # vfio-pci takes these IDs at boot, before the GPU's own driver.\n\
         options vfio-pci ids={}\n",
        gpu.primary.address,
        gpu.ids().join(",")
    );
    for d in HOST_DRIVERS {
        modprobe.push_str(&format!("softdep {d} pre: vfio-pci\n"));
    }
    let mut files = vec![
        FileChange {
            path: MODPROBE_FILE.into(),
            contents: modprobe,
        },
        FileChange {
            path: UDEV_FILE.into(),
            contents: "# Written by Cuvée: the kvm group opens VFIO groups, so QEMU runs as you.\n\
                       SUBSYSTEM==\"vfio\", GROUP=\"kvm\", MODE=\"0660\"\n"
                .into(),
        },
        FileChange {
            path: LIMITS_FILE.into(),
            contents:
                "# Written by Cuvée: QEMU locks all of a VM's RAM when it has a VFIO device.\n\
                       @kvm soft memlock unlimited\n\
                       @kvm hard memlock unlimited\n"
                    .into(),
        },
        FileChange {
            path: USER_SERVICE_FILE.into(),
            contents:
                "# Written by Cuvée: apps the desktop starts get the same locked-memory limit.\n\
                       [Service]\n\
                       LimitMEMLOCK=infinity\n"
                    .into(),
        },
    ];
    let mut commands = vec![
        "udevadm control --reload".to_string(),
        "udevadm trigger --subsystem-match=vfio".into(),
    ];
    match init {
        Initramfs::Mkinitcpio => {
            files.push(FileChange {
                path: "/etc/mkinitcpio.conf.d/cuvee-vfio.conf".into(),
                contents:
                    "# Written by Cuvée: vfio-pci in the initramfs, ahead of the GPU drivers.\n\
                           MODULES+=(vfio_pci vfio vfio_iommu_type1)\n"
                        .into(),
            });
            commands.push("mkinitcpio -P".into());
        }
        Initramfs::Dracut => {
            files.push(FileChange {
                path: "/etc/dracut.conf.d/cuvee-vfio.conf".into(),
                contents:
                    "# Written by Cuvée: vfio-pci in the initramfs, ahead of the GPU drivers.\n\
                           force_drivers+=\" vfio_pci vfio vfio_iommu_type1 \"\n"
                        .into(),
            });
            commands.push("dracut -f --regenerate-all".into());
        }
        Initramfs::InitramfsTools => {
            files.push(FileChange {
                path: "/etc/initramfs-tools/modules.d/cuvee-vfio".into(),
                contents: "vfio_pci\nvfio\nvfio_iommu_type1\n".into(),
            });
            commands.push("update-initramfs -u -k all".into());
        }
        Initramfs::Unknown => {}
    }
    commands.push("systemctl daemon-reload".into());
    let mut bind_now = vec!["modprobe vfio-pci".to_string()];
    for d in &gpu.functions {
        let a = &d.address;
        bind_now.push(format!(
            "echo vfio-pci > /sys/bus/pci/devices/{a}/driver_override"
        ));
        bind_now.push(format!(
            "if [ -e /sys/bus/pci/devices/{a}/driver ]; then echo {a} > /sys/bus/pci/devices/{a}/driver/unbind; fi"
        ));
        bind_now.push(format!("echo {a} > /sys/bus/pci/drivers_probe"));
    }
    HostPlan {
        files,
        commands,
        bind_now,
        kernel_params: missing_kernel_params(cpu, cmdline),
    }
}

/// Single-quotes `s` for sh.
pub fn sh_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=,@+".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

impl HostPlan {
    /// The plan as one sh script, to run as root (`pkexec sh -c`, `sudo sh -c`).
    /// `bind` adds the commands that move the card to vfio-pci now.
    pub fn script(&self, bind: bool) -> String {
        let mut s = String::from("set -e\n");
        for f in &self.files {
            if let Some(dir) = f.path.parent() {
                s.push_str(&format!("mkdir -p {}\n", sh_quote(&dir.to_string_lossy())));
            }
            s.push_str(&format!(
                "cat > {} <<'CUVEE_EOF'\n{}CUVEE_EOF\n",
                sh_quote(&f.path.to_string_lossy()),
                f.contents
            ));
        }
        for c in &self.commands {
            s.push_str(c);
            s.push('\n');
        }
        if bind {
            for c in &self.bind_now {
                s.push_str(c);
                s.push('\n');
            }
        }
        s
    }
}

/// The script that undoes [`HostPlan`]: removes Cuvée's files, rebuilds the initramfs.
pub fn undo_script(init: Initramfs) -> String {
    let mut s = String::from("set -e\n");
    for p in [MODPROBE_FILE, UDEV_FILE, LIMITS_FILE, USER_SERVICE_FILE] {
        s.push_str(&format!("rm -f {p}\n"));
    }
    match init {
        Initramfs::Mkinitcpio => {
            s.push_str("rm -f /etc/mkinitcpio.conf.d/cuvee-vfio.conf\nmkinitcpio -P\n")
        }
        Initramfs::Dracut => {
            s.push_str("rm -f /etc/dracut.conf.d/cuvee-vfio.conf\ndracut -f --regenerate-all\n")
        }
        Initramfs::InitramfsTools => s.push_str(
            "rm -f /etc/initramfs-tools/modules.d/cuvee-vfio\nupdate-initramfs -u -k all\n",
        ),
        Initramfs::Unknown => {}
    }
    s.push_str("udevadm control --reload\nsystemctl daemon-reload\n");
    s
}

/// The id prefix of every QEMU device Cuvée adds, so they can be found and taken out again.
pub const DEVICE_PREFIX: &str = "cuvee-gpu";

/// A free slot on q35's root bus: reims-ctl uses 0x1 to 0xa and 0x1b, q35 itself 0x1f.
const ROOT_PORT_ADDR: &str = "0x1c";

/// QEMU's arguments for the card: a PCIe root port of its own, and every function of the card
/// on it at the same function numbers, so macOS sees one multi-function GPU.
pub fn qemu_args(gpu: &Gpu) -> Vec<String> {
    let port = format!("{DEVICE_PREFIX}-port");
    let mut a = vec![
        "-device".to_string(),
        format!("pcie-root-port,id={port},bus=pcie.0,addr={ROOT_PORT_ADDR},chassis=28,slot=28"),
    ];
    let multi = gpu.functions.len() > 1;
    for d in &gpu.functions {
        let f = d.function();
        let mut dev = format!(
            "vfio-pci,host={},id={DEVICE_PREFIX}-{f},bus={port},addr=0x0.{f}",
            d.address
        );
        if f == 0 && multi {
            dev.push_str(",multifunction=on");
        }
        a.push("-device".into());
        a.push(dev);
    }
    a
}

/// `existing` without Cuvée's own `-device` pairs.
pub fn without_ours(existing: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(existing.len());
    let mut i = 0;
    while i < existing.len() {
        let ours = existing[i] == "-device"
            && existing
                .get(i + 1)
                .is_some_and(|v| v.contains(&format!("id={DEVICE_PREFIX}")));
        if ours {
            i += 2;
        } else {
            out.push(existing[i].clone());
            i += 1;
        }
    }
    out
}

/// A VM's settings Cuvée reads from reims-ctl's `vm.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmFiles {
    pub extra_args: Vec<String>,
    /// The OpenCore image, absolute.
    pub opencore: Option<PathBuf>,
    /// "qcow2" or "raw".
    pub opencore_format: String,
}

/// Reads `<dir>/vm.toml`, the file reims-ctl keeps a VM's settings in.
pub fn read_vm_files(dir: &Path) -> anyhow::Result<VmFiles> {
    use anyhow::Context;
    let file = dir.join("vm.toml");
    let text =
        std::fs::read_to_string(&file).with_context(|| format!("read {}", file.display()))?;
    parse_vm_toml(&text, dir)
}

pub fn parse_vm_toml(text: &str, dir: &Path) -> anyhow::Result<VmFiles> {
    let v: toml::Table = toml::from_str(text)?;
    let extra_args = v
        .get("extra_args")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let q35 = v.get("q35").and_then(|q| q.as_table());
    let opencore = q35
        .and_then(|q| q.get("opencore"))
        .and_then(|o| o.as_str())
        .map(|o| {
            let p = PathBuf::from(o);
            if p.is_absolute() {
                p
            } else {
                dir.join(p)
            }
        });
    let opencore_format = q35
        .and_then(|q| q.get("opencore_format"))
        .and_then(|o| o.as_str())
        .unwrap_or("qcow2")
        .to_string();
    Ok(VmFiles {
        extra_args,
        opencore,
        opencore_format,
    })
}

fn joined(args: &[String]) -> String {
    args.iter()
        .map(|a| sh_quote(a))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `reims-ctl set NAME --graphics vmware-svga --extra-args "…"`: the VM gets the card, and
/// keeps a QEMU window with the firmware's screen for OpenCore's picker and the installer.
/// macOS itself shows up on the monitor plugged into the card.
pub fn attach_command(name: &str, gpu: &Gpu, existing: &[String]) -> std::process::Command {
    let mut args = without_ours(existing);
    args.extend(qemu_args(gpu));
    let mut c = std::process::Command::new("reims-ctl");
    c.args(["set", name, "--graphics", "vmware-svga", "--extra-args"])
        .arg(joined(&args));
    c
}

/// Takes the card back out of the VM, keeping any other extra arguments.
pub fn detach_command(name: &str, existing: &[String]) -> std::process::Command {
    let args = without_ours(existing);
    let mut c = std::process::Command::new("reims-ctl");
    c.args(["set", name, "--graphics", "reims-vgpu", "--extra-args"])
        .arg(joined(&args));
    c
}

/// The host's GPUs, with what it takes to pass each, for `version` and `memory_gib`.
pub fn survey(version: Version, memory_gib: u32) -> (HostState, Vec<(Gpu, Assessment)>) {
    let sys = Path::new(SYSFS);
    let devices = pci::scan(sys);
    let host = host_state(sys, &devices);
    let list = gpus(&devices)
        .into_iter()
        .map(|g| {
            let a = assess(&g, version, memory_gib, &host);
            (g, a)
        })
        .collect();
    (host, list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pci::fake::Sysfs;

    fn host(displays: usize) -> HostState {
        HostState {
            iommu: true,
            cpu: CpuVendor::Amd,
            cmdline: "quiet rw".into(),
            memlock_bytes: None,
            displays,
        }
    }

    /// Yuri's desktop with an RX 580 added: the RTX 5070 drives the host, the RX 580 is spare.
    fn two_gpus(s: &Sysfs) {
        s.add(
            "0000:00:01.1",
            0x1022,
            0x1483,
            0x060400,
            Some("pcieport"),
            1,
            false,
        )
        .add(
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
        )
        .add(
            "0000:0c:00.0",
            0x1002,
            0x67df,
            0x030000,
            Some("vfio-pci"),
            23,
            false,
        )
        .add(
            "0000:0c:00.1",
            0x1002,
            0xaaf0,
            0x040300,
            Some("vfio-pci"),
            23,
            false,
        )
        .add(
            "0000:0b:00.0",
            0x1022,
            0x1484,
            0x060400,
            Some("pcieport"),
            23,
            false,
        );
    }

    #[test]
    fn groups_functions_and_ignores_bridges() {
        let s = Sysfs::new("gpus");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        assert_eq!(gpus.len(), 2);
        let rx = find(&gpus, "0c:00").unwrap();
        assert_eq!(rx.family, Family::Polaris);
        assert_eq!(rx.ids(), ["1002:67df", "1002:aaf0"]);
        assert!(rx.group_others.is_empty());
        assert_eq!(
            find(&gpus, "0000:09:00.0").unwrap().family,
            Family::NvidiaUnsupported
        );
    }

    #[test]
    fn spare_polaris_is_ready_for_sequoia() {
        let s = Sysfs::new("ready");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        let rx = find(&gpus, "0c:00").unwrap();
        let a = assess(rx, Version::Sequoia, 8, &host(2));
        // /dev/vfio/23 does not exist in the test, so only that remains.
        assert_eq!(a.findings, [Finding::VfioAccess(23)]);
        assert!(a.possible());
    }

    #[test]
    fn host_gpu_and_shared_group_are_blockers() {
        let s = Sysfs::new("block");
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
        )
        .add(
            "0000:0a:00.0",
            0x1022,
            0x43d5,
            0x0c0330,
            Some("xhci_hcd"),
            16,
            false,
        );
        let gpus = gpus(&pci::scan(s.path()));
        let a = assess(&gpus[0], Version::Sequoia, 8, &host(1));
        assert!(a
            .findings
            .contains(&Finding::Unsupported(Support::Unsupported)));
        assert!(a
            .findings
            .contains(&Finding::GroupShared(vec!["0000:0a:00.0".into()])));
        assert!(a.findings.contains(&Finding::HostScreen { only: true }));
        assert!(matches!(a.findings.last(), Some(Finding::NotBound(_))));
        assert_eq!(a.worst(), Level::Bad);
        assert!(!a.possible());
    }

    #[test]
    fn low_memlock_and_iommu_off_are_reported() {
        let s = Sysfs::new("memlock");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        let rx = find(&gpus, "0c:00").unwrap();
        let mut h = host(2);
        h.memlock_bytes = Some(8 << 20);
        h.iommu = false;
        h.cpu = CpuVendor::Intel;
        let a = assess(rx, Version::Monterey, 8, &h);
        assert_eq!(
            a.findings[0],
            Finding::IommuOff {
                missing_params: vec!["intel_iommu=on", "iommu=pt"]
            }
        );
        assert!(a.findings.contains(&Finding::Memlock {
            have: 8 << 20,
            need: 9 << 30
        }));
    }

    #[test]
    fn memlock_line_parses() {
        let l = "Limit                     Soft Limit           Hard Limit           Units\n\
                 Max locked memory         8388608              8388608              bytes\n";
        assert_eq!(parse_memlock(l), Some(Some(8388608)));
        let u = "Max locked memory         unlimited            unlimited            bytes\n";
        assert_eq!(parse_memlock(u), Some(None));
    }

    #[test]
    fn qemu_args_keep_function_numbers() {
        let s = Sysfs::new("qemu");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        let rx = find(&gpus, "0c:00").unwrap();
        let a = qemu_args(rx);
        assert_eq!(a.len(), 6);
        assert!(a[1].starts_with("pcie-root-port,id=cuvee-gpu-port,bus=pcie.0,addr=0x1c"));
        assert_eq!(
            a[3],
            "vfio-pci,host=0000:0c:00.0,id=cuvee-gpu-0,bus=cuvee-gpu-port,addr=0x0.0,multifunction=on"
        );
        assert_eq!(
            a[5],
            "vfio-pci,host=0000:0c:00.1,id=cuvee-gpu-1,bus=cuvee-gpu-port,addr=0x0.1"
        );
    }

    #[test]
    fn attach_keeps_other_args_and_replaces_ours() {
        let s = Sysfs::new("attach");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        let rx = find(&gpus, "0c:00").unwrap();
        let mut existing = vec!["-boot".to_string(), "menu=on".into()];
        existing.extend(qemu_args(rx));
        let c = attach_command("mac", rx, &existing);
        let args: Vec<String> = c.get_args().map(|a| a.to_string_lossy().into()).collect();
        assert_eq!(
            &args[..5],
            ["set", "mac", "--graphics", "vmware-svga", "--extra-args"]
        );
        assert!(args[5].starts_with("-boot menu=on -device pcie-root-port"));
        assert_eq!(args[5].matches("pcie-root-port").count(), 1);
        let d = detach_command("mac", &existing);
        let args: Vec<String> = d.get_args().map(|a| a.to_string_lossy().into()).collect();
        assert_eq!(args[5], "-boot menu=on");
    }

    #[test]
    fn host_plan_writes_ids_and_initramfs() {
        let s = Sysfs::new("plan");
        two_gpus(&s);
        let gpus = gpus(&pci::scan(s.path()));
        let rx = find(&gpus, "0c:00").unwrap();
        let p = host_plan(
            rx,
            Initramfs::Mkinitcpio,
            CpuVendor::Intel,
            "quiet iommu=pt",
        );
        assert!(p.files[0]
            .contents
            .contains("options vfio-pci ids=1002:67df,1002:aaf0"));
        assert!(p.files[0].contents.contains("softdep amdgpu pre: vfio-pci"));
        assert!(p.commands.contains(&"mkinitcpio -P".to_string()));
        assert_eq!(p.kernel_params, ["intel_iommu=on"]);
        let script = p.script(true);
        assert!(script.contains("cat > /etc/modprobe.d/cuvee-vfio.conf <<'CUVEE_EOF'"));
        assert!(
            script.contains("echo vfio-pci > /sys/bus/pci/devices/0000:0c:00.0/driver_override")
        );
        assert!(!p.script(false).contains("driver_override"));
    }

    #[test]
    fn reads_reims_ctl_vm_toml() {
        let t = r#"
name = "mac"
extra_args = ["-boot", "menu=on"]
[q35]
opencore = "OpenCore.qcow2"
"#;
        let f = parse_vm_toml(t, Path::new("/vms/mac")).unwrap();
        assert_eq!(f.extra_args, ["-boot", "menu=on"]);
        assert_eq!(f.opencore, Some(PathBuf::from("/vms/mac/OpenCore.qcow2")));
        assert_eq!(f.opencore_format, "qcow2");
    }

    #[test]
    fn quoting() {
        assert_eq!(
            sh_quote("vfio-pci,host=0000:0c:00.0"),
            "vfio-pci,host=0000:0c:00.0"
        );
        assert_eq!(sh_quote("a b"), "'a b'");
        assert_eq!(sh_quote("it's"), r"'it'\''s'");
    }
}
