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
//! The image is edited with mtools (`mcopy`) on a raw copy; a qcow2 image goes through
//! `qemu-img convert` both ways. The original is kept beside it as `<name>.cuvee-bak`.

use crate::macos::Version;
use anyhow::{bail, ensure, Context, Result};
use plist::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// The byte offset of the first partition of a GPT disk, from its first 34 sectors.
pub fn esp_offset(head: &[u8]) -> Result<u64> {
    const SECTOR: usize = 512;
    ensure!(head.len() >= SECTOR * 3, "image too small for a GPT");
    let hdr = &head[SECTOR..SECTOR * 2];
    ensure!(&hdr[..8] == b"EFI PART", "no GPT header");
    let le64 = |b: &[u8]| u64::from_le_bytes(b[..8].try_into().unwrap());
    let le32 = |b: &[u8]| u32::from_le_bytes(b[..4].try_into().unwrap());
    let entries_lba = le64(&hdr[72..]) as usize;
    let count = le32(&hdr[80..]) as usize;
    let size = le32(&hdr[84..]) as usize;
    ensure!(size >= 128, "GPT entry size {size}");
    for i in 0..count.min(128) {
        let at = entries_lba * SECTOR + i * size;
        let Some(e) = head.get(at..at + size) else {
            break;
        };
        if e[..16].iter().any(|b| *b != 0) {
            return Ok(le64(&e[32..]) * SECTOR as u64);
        }
    }
    bail!("the GPT has no partition")
}

/// Where `qemu-img` is: beside this install's QEMU (crate::paths), else the one on PATH.
pub fn qemu_img() -> PathBuf {
    crate::paths::qemu_binary()
        .map(|q| q.with_file_name("qemu-img"))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("qemu-img"))
}

fn run(c: &mut Command) -> Result<()> {
    let what = format!("{c:?}");
    let out = c
        .env("MTOOLS_SKIP_CHECK", "1")
        .output()
        .with_context(|| format!("run {what}"))?;
    ensure!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(())
}

fn with_suffix(p: &Path, suffix: &str) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// What [`prepare`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    AlreadyDone,
    Patched { backup: PathBuf },
}

/// Sets up the OpenCore image at `image` (format "qcow2" or "raw") for root patching on `v`.
/// The VM must be off.
pub fn prepare(image: &Path, format: &str, v: Version) -> Result<Prepared> {
    let dir = image.parent().context("image has no directory")?;
    let raw = if format == "qcow2" {
        let raw = dir.join(".cuvee-opencore.raw");
        run(Command::new(qemu_img())
            .args(["convert", "-f", "qcow2", "-O", "raw"])
            .arg(image)
            .arg(&raw))?;
        raw
    } else {
        image.to_path_buf()
    };
    let result = patch_raw(&raw, v);
    let done = match result {
        Ok(false) => Prepared::AlreadyDone,
        Ok(true) => {
            let backup = with_suffix(image, ".cuvee-bak");
            if !backup.exists() {
                if format == "qcow2" {
                    std::fs::copy(image, &backup)
                } else {
                    // The raw image was edited in place: the backup is taken in patch_raw.
                    Ok(0)
                }
                .with_context(|| format!("back up {}", image.display()))?;
            }
            if format == "qcow2" {
                let tmp = with_suffix(image, ".cuvee-new");
                run(Command::new(qemu_img())
                    .args(["convert", "-f", "raw", "-O", "qcow2"])
                    .arg(&raw)
                    .arg(&tmp))?;
                std::fs::rename(&tmp, image)?;
            }
            Prepared::Patched { backup }
        }
        Err(e) => {
            if format == "qcow2" {
                let _ = std::fs::remove_file(&raw);
            }
            return Err(e);
        }
    };
    if format == "qcow2" {
        let _ = std::fs::remove_file(&raw);
    }
    Ok(done)
}

/// Patches config.plist inside the raw image; false if it was already patched.
fn patch_raw(raw: &Path, v: Version) -> Result<bool> {
    use std::io::Read;
    let mut head = vec![0u8; 512 * 34];
    std::fs::File::open(raw)?.read_exact(&mut head)?;
    let at = format!("{}@@{}", raw.display(), esp_offset(&head)?);
    let tmp = std::env::temp_dir().join(format!("cuvee-config-{}.plist", std::process::id()));
    run(Command::new("mcopy")
        .args(["-n", "-i", &at, "::/EFI/OC/config.plist"])
        .arg(&tmp))
    .context("mcopy (from mtools) could not read EFI/OC/config.plist; is mtools installed?")?;
    let bytes = std::fs::read(&tmp)?;
    if is_patched(&bytes, v) {
        let _ = std::fs::remove_file(&tmp);
        return Ok(false);
    }
    // A raw image is edited in place: keep its original first.
    if !raw.file_name().is_some_and(|n| n == ".cuvee-opencore.raw") {
        let backup = with_suffix(raw, ".cuvee-bak");
        if !backup.exists() {
            std::fs::copy(raw, &backup)?;
        }
    }
    std::fs::write(&tmp, patch_config(&bytes, v)?)?;
    let r = run(Command::new("mcopy")
        .args(["-o", "-i", &at])
        .arg(&tmp)
        .arg("::/EFI/OC/config.plist"));
    let _ = std::fs::remove_file(&tmp);
    r.map(|_| true)
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

    #[test]
    fn finds_the_first_partition() {
        let mut head = vec![0u8; 512 * 34];
        head[512..520].copy_from_slice(b"EFI PART");
        head[512 + 72..512 + 80].copy_from_slice(&2u64.to_le_bytes());
        head[512 + 80..512 + 84].copy_from_slice(&128u32.to_le_bytes());
        head[512 + 84..512 + 88].copy_from_slice(&128u32.to_le_bytes());
        let e = 1024;
        head[e] = 0x28; // any non-zero type GUID
        head[e + 32..e + 40].copy_from_slice(&2048u64.to_le_bytes());
        assert_eq!(esp_offset(&head).unwrap(), 2048 * 512);
        assert!(esp_offset(&vec![0u8; 512 * 34]).is_err());
    }
}
