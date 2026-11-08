//! Reading and editing the `EFI/OC/config.plist` inside a VM's OpenCore image.
//!
//! reims-util writes the image: a qcow2 (or raw) disk with a GPT and a FAT32 EFI system
//! partition. The file is read and written with mtools (`mcopy`) on a raw copy; a qcow2 image
//! goes through `qemu-img convert` both ways. Before the first change, the original image is
//! kept beside it as `<name>.cuvee-bak`.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{bail, ensure, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

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

pub(crate) fn with_suffix(p: &Path, suffix: &str) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// A file name in the temporary directory no other call of this process uses.
fn temp_file(what: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    std::env::temp_dir().join(format!(
        "cuvee-{what}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ))
}

/// The raw disk to work on: the image itself, or a raw copy of a qcow2 one.
struct Raw {
    path: PathBuf,
    copy: bool,
}

impl Raw {
    /// `shared` reads an image a running VM holds (qemu-img's `-U`): fine for reading only.
    fn open(image: &Path, format: &str, shared: bool) -> Result<Raw> {
        if format != "qcow2" {
            return Ok(Raw {
                path: image.to_path_buf(),
                copy: false,
            });
        }
        let dir = image.parent().context("image has no directory")?;
        let path = dir.join(format!(".cuvee-opencore-{}.raw", std::process::id()));
        let mut c = Command::new(qemu_img());
        c.arg("convert");
        if shared {
            c.arg("-U");
        }
        run(c.args(["-f", "qcow2", "-O", "raw"]).arg(image).arg(&path))?;
        Ok(Raw { path, copy: true })
    }

    /// mtools' `image@@offset` for the EFI system partition.
    fn esp(&self) -> Result<String> {
        use std::io::Read;
        let mut head = vec![0u8; 512 * 34];
        std::fs::File::open(&self.path)?.read_exact(&mut head)?;
        Ok(format!("{}@@{}", self.path.display(), esp_offset(&head)?))
    }

    fn read_config(&self) -> Result<Vec<u8>> {
        let tmp = temp_file("config.plist");
        let r = run(Command::new("mcopy")
            .args(["-n", "-i", &self.esp()?, "::/EFI/OC/config.plist"])
            .arg(&tmp))
        .context("mcopy (from mtools) could not read EFI/OC/config.plist; is mtools installed?")
        .and_then(|()| Ok(std::fs::read(&tmp)?));
        let _ = std::fs::remove_file(&tmp);
        r
    }

    fn write_config(&self, bytes: &[u8]) -> Result<()> {
        let tmp = temp_file("config.plist");
        std::fs::write(&tmp, bytes)?;
        let r = self.esp().and_then(|at| {
            run(Command::new("mcopy")
                .args(["-o", "-i", &at])
                .arg(&tmp)
                .arg("::/EFI/OC/config.plist"))
        });
        let _ = std::fs::remove_file(&tmp);
        r
    }
}

impl Drop for Raw {
    fn drop(&mut self) {
        if self.copy {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// The image's `EFI/OC/config.plist`. `format` is "qcow2" or "raw". The VM may be running.
pub fn read_config(image: &Path, format: &str) -> Result<Vec<u8>> {
    Raw::open(image, format, true)?.read_config()
}

/// Rewrites the image's `config.plist` with what `change` makes of it. `change` returns None
/// when there is nothing to change, and so does this; otherwise the backup's path. The VM must
/// be off.
pub fn edit_config(
    image: &Path,
    format: &str,
    change: impl FnOnce(&[u8]) -> Result<Option<Vec<u8>>>,
) -> Result<Option<PathBuf>> {
    let raw = Raw::open(image, format, false)?;
    let Some(new) = change(&raw.read_config()?)? else {
        return Ok(None);
    };
    let backup = with_suffix(image, ".cuvee-bak");
    if !backup.exists() {
        std::fs::copy(image, &backup).with_context(|| format!("back up {}", image.display()))?;
    }
    raw.write_config(&new)?;
    if raw.copy {
        let tmp = with_suffix(image, ".cuvee-new");
        run(Command::new(qemu_img())
            .args(["convert", "-f", "raw", "-O", "qcow2"])
            .arg(&raw.path)
            .arg(&tmp))?;
        std::fs::rename(&tmp, image)?;
    }
    Ok(Some(backup))
}

#[cfg(test)]
mod tests {
    use super::*;

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
