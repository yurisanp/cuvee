//! Getting a VM's OpenCore ready for OpenCore Legacy Patcher's root patches.
//!
//! macOS 13 to 15 dropped the drivers of NVIDIA Kepler and AMD GCN 1–3 cards. OCLP's
//! "Post-Install Root Patch", run inside the guest, puts them back. It writes to the system
//! volume, so it needs System Integrity Protection partly off (including
//! `CSR_ALLOW_UNAUTHENTICATED_ROOT`, 0x800) and, on 13 and newer, AMFI relaxed (`amfi=0x80`).
//! reims-util's OpenCore turns SIP off as 0x67, without 0x800, and its NVRAM settings win at
//! every boot, so they are changed in the image's `config.plist` itself.
//!
//! Cuvée ships none of OCLP. The person downloads OCLP from its own GitHub releases inside
//! macOS; OCLP's license and its support terms are its own (OCLP supports real Macs only).
//!
//! The image is edited through [`crate::opencore`]; the original is kept beside it as
//! `<name>.cuvee-bak`.

use crate::macos::Version;
use anyhow::{Context, Result};
use plist::Value;
use std::path::{Path, PathBuf};

/// Apple's NVRAM GUID, where OpenCore keeps boot-args and csr-active-config.
const APPLE_NVRAM: &str = "7C436110-AB2A-4BBB-A880-FE41995C9F82";

/// SIP for root patching: reims-util's 0x67 plus CSR_ALLOW_UNAUTHENTICATED_ROOT (0x800).
pub const CSR: u32 = 0x867;

/// Boot arguments OCLP's root patches need on `v`, besides reims-util's own.
pub fn boot_args(v: Version) -> &'static [&'static str] {
    use Version::*;
    match v {
        Ventura | Sonoma | Sequoia => &["amfi=0x80", "ipc_control_port_options=0"],
        _ => &[],
    }
}

fn nvram_mut(root: &mut Value) -> Result<&mut plist::Dictionary> {
    root.as_dictionary_mut()
        .and_then(|d| d.get_mut("NVRAM"))
        .and_then(Value::as_dictionary_mut)
        .and_then(|d| d.get_mut("Add"))
        .and_then(Value::as_dictionary_mut)
        .and_then(|d| d.get_mut(APPLE_NVRAM))
        .and_then(Value::as_dictionary_mut)
        .context("config.plist has no NVRAM > Add > Apple's GUID")
}

/// `config.plist` with SIP and the boot arguments set for root patching on `v`.
pub fn patch_config(bytes: &[u8], v: Version) -> Result<Vec<u8>> {
    let mut root = Value::from_reader(std::io::Cursor::new(bytes)).context("read config.plist")?;
    let nvram = nvram_mut(&mut root)?;
    let mut args: Vec<String> = nvram
        .get("boot-args")
        .and_then(Value::as_string)
        .unwrap_or("")
        .split_whitespace()
        .map(String::from)
        .collect();
    for a in boot_args(v) {
        let key = a.split('=').next().unwrap_or(a);
        args.retain(|x| x.split('=').next() != Some(key));
        args.push(a.to_string());
    }
    nvram.insert("boot-args".into(), Value::String(args.join(" ")));
    nvram.insert(
        "csr-active-config".into(),
        Value::Data(CSR.to_le_bytes().to_vec()),
    );
    let mut out = Vec::new();
    root.to_writer_xml(&mut out).context("write config.plist")?;
    Ok(out)
}

/// Whether `config.plist` is already set up for root patching on `v`.
pub fn is_patched(bytes: &[u8], v: Version) -> bool {
    let Ok(mut root) = Value::from_reader(std::io::Cursor::new(bytes)) else {
        return false;
    };
    let Ok(nvram) = nvram_mut(&mut root) else {
        return false;
    };
    let csr_ok = nvram
        .get("csr-active-config")
        .and_then(Value::as_data)
        .is_some_and(|d| d == CSR.to_le_bytes());
    let have: Vec<&str> = nvram
        .get("boot-args")
        .and_then(Value::as_string)
        .unwrap_or("")
        .split_whitespace()
        .collect();
    csr_ok && boot_args(v).iter().all(|a| have.contains(a))
}

pub use crate::opencore::{esp_offset, qemu_img};

/// What [`prepare`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    AlreadyDone,
    Patched { backup: PathBuf },
}

/// Sets up the OpenCore image at `image` (format "qcow2" or "raw") for root patching on `v`.
/// The VM must be off.
pub fn prepare(image: &Path, format: &str, v: Version) -> Result<Prepared> {
    let done = crate::opencore::edit_config(image, format, |bytes| {
        if is_patched(bytes, v) {
            Ok(None)
        } else {
            patch_config(bytes, v).map(Some)
        }
    })?;
    Ok(match done {
        None => Prepared::AlreadyDone,
        Some(backup) => Prepared::Patched { backup },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>NVRAM</key><dict><key>Add</key><dict>
<key>7C436110-AB2A-4BBB-A880-FE41995C9F82</key><dict>
<key>boot-args</key><string>keepsyms=1 tlbto_us=0 vti=9 amfi=0x0</string>
<key>csr-active-config</key><data>ZwAAAA==</data>
</dict></dict></dict>
</dict></plist>"#;

    #[test]
    fn patches_sip_and_boot_args() {
        let out = patch_config(CONFIG.as_bytes(), Version::Sonoma).unwrap();
        let text = String::from_utf8(out.clone()).unwrap();
        assert!(text.contains("keepsyms=1 tlbto_us=0 vti=9 amfi=0x80 ipc_control_port_options=0"));
        assert!(!text.contains("amfi=0x0"));
        assert!(is_patched(&out, Version::Sonoma));
        assert!(!is_patched(CONFIG.as_bytes(), Version::Sonoma));
        // Patching twice changes nothing.
        assert_eq!(patch_config(&out, Version::Sonoma).unwrap(), out);
    }

    #[test]
    fn monterey_needs_only_sip() {
        let out = patch_config(CONFIG.as_bytes(), Version::Monterey).unwrap();
        assert!(String::from_utf8_lossy(&out).contains("vti=9 amfi=0x0<"));
        assert!(is_patched(&out, Version::Monterey));
    }
}
