//! The host's Vulkan drivers: the ICD manifests the Vulkan loader would find, and a
//! `vulkaninfo` run that does not depend on the desktop session.
//!
//! A driver is found by its manifest, the way the loader finds it, so the check is the same
//! on every distribution whatever the package is called (nvidia-utils on Arch, nvidia-driver
//! on Debian, xorg-x11-drv-nvidia on Fedora, nvidia-gl-G06 on openSUSE).
//!
//! `vulkaninfo` runs without `DISPLAY` and `WAYLAND_DISPLAY`: with them it also creates
//! window surfaces, and when it cannot reach the X server it crashes, which said nothing
//! about the driver.

use crate::host::GpuVendor;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One ICD manifest: the file and the library it points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icd {
    pub path: PathBuf,
    pub library: String,
}

impl Icd {
    /// The GPU vendor this driver is for, from the manifest's name and library.
    pub fn vendor(&self) -> GpuVendor {
        let file = self
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let lib = Path::new(&self.library)
            .file_name()
            .map(|f| f.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        vendor_of(&file, &lib)
    }
}

fn vendor_of(file: &str, lib: &str) -> GpuVendor {
    let has = |s: &str| file.contains(s) || lib.contains(s);
    if has("nvidia") {
        GpuVendor::Nvidia
    } else if has("radeon") || has("amdvlk") || has("amd_icd") || has("amdgpu") {
        GpuVendor::Amd
    } else if has("intel") {
        GpuVendor::Intel
    } else {
        GpuVendor::Other
    }
}

fn split_paths(v: &std::ffi::OsStr) -> Vec<PathBuf> {
    std::env::split_paths(v)
        .filter(|p| p.is_absolute())
        .collect()
}

fn env_dirs(var: &str, default: &str) -> Vec<PathBuf> {
    match std::env::var_os(var).filter(|v| !v.is_empty()) {
        Some(v) => split_paths(&v),
        None => split_paths(std::ffi::OsStr::new(default)),
    }
}

fn home_dir(var: &str, fallback: &str) -> Option<PathBuf> {
    if let Some(v) = std::env::var_os(var).filter(|v| Path::new(v).is_absolute()) {
        return Some(PathBuf::from(v));
    }
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(fallback))
}

/// Where the loader looks for ICD manifests, in its order. `VK_DRIVER_FILES` (and the older
/// `VK_ICD_FILENAMES`) replace the search, as they do for the loader.
pub fn search_paths() -> Vec<PathBuf> {
    for var in ["VK_DRIVER_FILES", "VK_ICD_FILENAMES"] {
        if let Some(v) = std::env::var_os(var).filter(|v| !v.is_empty()) {
            return split_paths(&v);
        }
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.extend(home_dir("XDG_CONFIG_HOME", ".config"));
    dirs.extend(env_dirs("XDG_CONFIG_DIRS", "/etc/xdg"));
    dirs.push("/etc".into());
    dirs.extend(home_dir("XDG_DATA_HOME", ".local/share"));
    dirs.extend(env_dirs("XDG_DATA_DIRS", "/usr/local/share:/usr/share"));
    // Where the distributions' packages put them, even if XDG_DATA_DIRS leaves it out.
    dirs.push("/usr/share".into());
    let mut out: Vec<PathBuf> = Vec::new();
    for d in dirs {
        let p = d.join("vulkan/icd.d");
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The library a manifest names, or None if the text is not an ICD manifest.
pub fn parse_manifest(text: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    v.get("ICD")?
        .get("library_path")?
        .as_str()
        .map(str::to_string)
}

fn read_icd(path: &Path) -> Option<Icd> {
    let text = std::fs::read_to_string(path).ok()?;
    Some(Icd {
        path: path.to_path_buf(),
        library: parse_manifest(&text)?,
    })
}

/// Every ICD manifest the loader would find.
pub fn icds() -> Vec<Icd> {
    let mut out: Vec<Icd> = Vec::new();
    for p in search_paths() {
        let files: Vec<PathBuf> = if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(&p)
                .map(|rd| {
                    rd.flatten()
                        .map(|e| e.path())
                        .filter(|f| f.extension().is_some_and(|x| x == "json"))
                        .collect()
                })
                .unwrap_or_default();
            v.sort();
            v
        } else {
            vec![p]
        };
        for f in files {
            if let Some(icd) = read_icd(&f) {
                if !out.iter().any(|o| o.path == icd.path) {
                    out.push(icd);
                }
            }
        }
    }
    out
}

/// Whether a Vulkan driver for `vendor` is installed.
pub fn has_driver(vendor: GpuVendor) -> bool {
    icds().iter().any(|i| i.vendor() == vendor)
}

/// What `vulkaninfo --summary` said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// It listed a real GPU (not only a CPU renderer such as llvmpipe).
    Gpu,
    /// It ran but found no GPU, or it failed.
    NoGpu,
}

/// Whether the summary lists a GPU device.
pub fn summary_has_gpu(summary: &str) -> bool {
    summary.lines().any(|l| {
        let l = l.trim();
        l.starts_with("deviceType")
            && (l.contains("DISCRETE_GPU")
                || l.contains("INTEGRATED_GPU")
                || l.contains("VIRTUAL_GPU"))
    })
}

/// The `vulkaninfo --summary` command, without the session's display.
pub fn summary_command() -> Option<Command> {
    let bin = crate::paths::which("vulkaninfo")?;
    let mut cmd = Command::new(bin);
    cmd.arg("--summary")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY");
    Some(cmd)
}

/// Runs `vulkaninfo --summary`. None if it is not installed.
pub fn probe() -> Option<Probe> {
    let out = summary_command()?.output();
    Some(match out {
        Ok(o) if o.status.success() && summary_has_gpu(&String::from_utf8_lossy(&o.stdout)) => {
            Probe::Gpu
        }
        _ => Probe::NoGpu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icd(path: &str, lib: &str) -> Icd {
        Icd {
            path: path.into(),
            library: lib.into(),
        }
    }

    #[test]
    fn vendors_from_manifests() {
        // Arch, Debian and Ubuntu.
        let n = icd(
            "/usr/share/vulkan/icd.d/nvidia_icd.json",
            "libGLX_nvidia.so.0",
        );
        assert_eq!(n.vendor(), GpuVendor::Nvidia);
        // Fedora (RPM Fusion) and openSUSE name it by arch.
        let n = icd(
            "/usr/share/vulkan/icd.d/nvidia_icd.x86_64.json",
            "libGLX_nvidia.so.0",
        );
        assert_eq!(n.vendor(), GpuVendor::Nvidia);
        // NVIDIA's .run installer, under /etc.
        let n = icd("/etc/vulkan/icd.d/nvidia_icd.json", "libGLX_nvidia.so.0");
        assert_eq!(n.vendor(), GpuVendor::Nvidia);
        // A manifest named otherwise still points at NVIDIA's library.
        let n = icd(
            "/usr/share/vulkan/icd.d/vendor.json",
            "/usr/lib64/libGLX_nvidia.so.0",
        );
        assert_eq!(n.vendor(), GpuVendor::Nvidia);
        let a = icd(
            "/usr/share/vulkan/icd.d/radeon_icd.x86_64.json",
            "/usr/lib/libvulkan_radeon.so",
        );
        assert_eq!(a.vendor(), GpuVendor::Amd);
        let a = icd(
            "/usr/share/vulkan/icd.d/amd_icd64.json",
            "/usr/lib/amdvlk64.so",
        );
        assert_eq!(a.vendor(), GpuVendor::Amd);
        let i = icd(
            "/usr/share/vulkan/icd.d/intel_icd.x86_64.json",
            "/usr/lib/libvulkan_intel.so",
        );
        assert_eq!(i.vendor(), GpuVendor::Intel);
        let i = icd(
            "/usr/share/vulkan/icd.d/intel_hasvk_icd.x86_64.json",
            "/usr/lib/libvulkan_intel_hasvk.so",
        );
        assert_eq!(i.vendor(), GpuVendor::Intel);
        let l = icd(
            "/usr/share/vulkan/icd.d/lvp_icd.x86_64.json",
            "/usr/lib/libvulkan_lvp.so",
        );
        assert_eq!(l.vendor(), GpuVendor::Other);
    }

    #[test]
    fn manifests_parse() {
        let nvidia = r#"{
            "file_format_version" : "1.0.1",
            "ICD": { "library_path": "libGLX_nvidia.so.0", "api_version" : "1.4.357" }
        }"#;
        assert_eq!(
            parse_manifest(nvidia).as_deref(),
            Some("libGLX_nvidia.so.0")
        );
        assert_eq!(parse_manifest("{\"layer\": {}}"), None);
        assert_eq!(parse_manifest("not json"), None);
    }

    #[test]
    fn summary_needs_a_gpu() {
        let nvidia = "Devices:\n========\nGPU0:\n\tapiVersion = 1.4.357\n\tdeviceType = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU\n\tdeviceName = NVIDIA GeForce RTX 5070\n";
        assert!(summary_has_gpu(nvidia));
        let cpu = "Devices:\nGPU0:\n\tdeviceType = PHYSICAL_DEVICE_TYPE_CPU\n\tdeviceName = llvmpipe (LLVM 19.1.7, 256 bits)\n";
        assert!(!summary_has_gpu(cpu));
        assert!(!summary_has_gpu(""));
    }
}
