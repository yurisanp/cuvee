//! The Mac a VM presents: its model, serial number, logic board serial (MLB), system UUID and
//! ROM, which OpenCore writes into SMBIOS and NVRAM. GenSMBIOS and the Olarila guide make the
//! same five values with macserial for a real hackintosh.
//!
//! Cuvée generates none of them itself: reims-util does, with its port of OpenCorePkg's
//! macserial tables (BSD-3-Clause), every time it makes an OpenCore image. reims-ctl already
//! has it make one with a new identity for each VM it creates. What Cuvée adds:
//!
//! - [`stage`]: an image made before the VM, so the "Nova máquina" review shows the identity
//!   the VM will have, and can make another; `reims-ctl create --opencore` then copies it.
//! - [`generate`]: an identity alone, as GenSMBIOS prints one (`cuvee smbios generate`).
//! - [`read_vm`] and [`renew_vm`]: a VM's identity, and a new one for it, written into its own
//!   image's `config.plist` so nothing else in it changes (`cuvee smbios show` and `new`).
//!
//! The ROM is the VM's network card's address, as a Mac's ROM is its first card's, so a new
//! identity comes with a new MAC address too.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

use crate::macos::{Model, Version};
use anyhow::{bail, ensure, Context, Result};
use plist::Value;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Apple's site that tells whether a serial number belongs to a Mac it sold. "We're unable to
/// check coverage for this serial number" is what a generated serial should get.
pub const CHECK_COVERAGE_URL: &str = "https://checkcoverage.apple.com";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub model: String,
    pub serial: String,
    pub mlb: String,
    pub uuid: String,
    /// Six bytes, as 12 hex digits.
    pub rom: String,
}

impl Identity {
    /// The ROM as a MAC address, `52:54:00:12:34:56`.
    pub fn rom_as_mac(&self) -> String {
        self.rom
            .as_bytes()
            .chunks(2)
            .map(|c| String::from_utf8_lossy(c).to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// A new MAC address for a VM's network card: QEMU's prefix, 52:54:00, which is locally
/// administered and unicast, and three random bytes.
pub fn random_mac() -> Result<String> {
    let mut b = [0u8; 3];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut b))
        .context("read /dev/urandom")?;
    Ok(format!("52:54:00:{:02x}:{:02x}:{:02x}", b[0], b[1], b[2]))
}

/// What `reims-util opencore --json` prints last: `{"event":"done","result":{..,"smbios":{..}}}`
/// or `{"event":"error","message":".."}`.
pub fn parse_reims_util(stdout: &str) -> Result<Identity> {
    #[derive(Deserialize)]
    struct Line {
        event: String,
        message: Option<String>,
        result: Option<Done>,
    }
    #[derive(Deserialize)]
    struct Done {
        smbios: Identity,
    }
    for l in stdout.lines().rev() {
        let Ok(line) = serde_json::from_str::<Line>(l) else {
            continue;
        };
        match line.event.as_str() {
            "done" => {
                return line
                    .result
                    .map(|r| r.smbios)
                    .context("reims-util gave no SMBIOS")
            }
            "error" => bail!("reims-util: {}", line.message.unwrap_or_default()),
            _ => {}
        }
    }
    bail!("reims-util printed no result")
}

/// Has reims-util make an OpenCore image at `image` presenting a new `model`, with `mac` as
/// its ROM; OVMF goes beside it when `ovmf`. Returns the identity it made.
fn make_image(image: &Path, model: Model, mac: &str, ovmf: bool) -> Result<Identity> {
    let util = crate::paths::reims_util()
        .context("reims-util is not installed: run cuvee install, which installs it")?;
    let mut c = std::process::Command::new(&util);
    c.args(["opencore", "--json", "-o"])
        .arg(image)
        .args(["--model", model.name(), "--rom", mac]);
    if !ovmf {
        c.arg("--no-ovmf");
    }
    let out = c
        .output()
        .with_context(|| format!("run {}", util.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let id = parse_reims_util(&stdout).with_context(|| {
        format!(
            "reims-util opencore failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )
    })?;
    ensure!(
        out.status.success(),
        "reims-util opencore failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(id)
}

/// A new identity for `model`, with `mac` (or a new address) as its ROM, and no VM. reims-util
/// only makes one along with an image, so the image is made in a temporary directory and
/// dropped; it takes a moment and no download once reims-util has its files cached.
pub fn generate(model: Model, mac: Option<&str>) -> Result<Identity> {
    let mac = match mac {
        Some(m) => m.to_string(),
        None => random_mac()?,
    };
    let dir = unique_dir(&std::env::temp_dir(), "cuvee-smbios")?;
    let r = make_image(&dir.join("opencore.img"), model, &mac, false);
    let _ = std::fs::remove_dir_all(&dir);
    r
}

fn unique_dir(parent: &Path, prefix: &str) -> Result<PathBuf> {
    let mut b = [0u8; 4];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut b))
        .context("read /dev/urandom")?;
    let dir = parent.join(format!(
        "{prefix}-{}-{:08x}",
        std::process::id(),
        u32::from_be_bytes(b)
    ));
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

/// An OpenCore image, with OVMF beside it, made for a VM not created yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    pub dir: PathBuf,
    /// `<dir>/opencore.qcow2`, for `reims-ctl create --opencore`.
    pub image: PathBuf,
    /// The network card's address, the identity's ROM, for `reims-ctl create --mac`.
    pub mac: String,
    pub identity: Identity,
}

/// Where staged images go: `~/.cache/cuvee/nova`.
pub fn staging_dir() -> PathBuf {
    crate::paths::cache_dir().join("nova")
}

/// Makes an image presenting a new `model` in its own directory under [`staging_dir`].
pub fn stage(model: Model) -> Result<Staged> {
    let dir = unique_dir(&staging_dir(), "vm")?;
    let image = dir.join("opencore.qcow2");
    let mac = random_mac()?;
    match make_image(&image, model, &mac, true) {
        Ok(identity) => Ok(Staged {
            dir,
            image,
            mac,
            identity,
        }),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            Err(e)
        }
    }
}

impl Staged {
    /// Removes the image, once reims-ctl has copied it or another replaced it.
    pub fn discard(&self) {
        if self.dir.starts_with(staging_dir()) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
}

/// Removes every staged image, such as those of a "Nova máquina" left without creating.
pub fn clear_staged() {
    let _ = std::fs::remove_dir_all(staging_dir());
}

/// The model reims-util is asked for: `name` if given (and it runs `v`), else the one that
/// suits `v`.
pub fn pick_model(name: Option<&str>, v: Option<Version>) -> Result<Model> {
    let Some(name) = name else {
        return Ok(v.map(|v| v.model()).unwrap_or(Model::IMac20_1));
    };
    let m = Model::parse(name).with_context(|| {
        format!(
            "unknown model {name:?}: {} are known",
            Model::ALL.map(|m| m.name()).join(" and ")
        )
    })?;
    if let Some(v) = v {
        ensure!(
            m.runs(v),
            "the {} does not run macOS {} {}",
            m.name(),
            v.number(),
            v.name()
        );
    }
    Ok(m)
}

fn generic(root: &Value) -> Option<&plist::Dictionary> {
    root.as_dictionary()?
        .get("PlatformInfo")?
        .as_dictionary()?
        .get("Generic")?
        .as_dictionary()
}

/// The identity in a `config.plist`: PlatformInfo > Generic.
pub fn read_config(bytes: &[u8]) -> Result<Identity> {
    let root = Value::from_reader(std::io::Cursor::new(bytes)).context("read config.plist")?;
    let g = generic(&root).context("config.plist has no PlatformInfo > Generic")?;
    let text = |key: &str| {
        g.get(key)
            .and_then(Value::as_string)
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
            .with_context(|| format!("config.plist's PlatformInfo > Generic has no {key}"))
    };
    let rom = g
        .get("ROM")
        .and_then(Value::as_data)
        .filter(|d| d.len() == 6)
        .context("config.plist's PlatformInfo > Generic has no six-byte ROM")?;
    Ok(Identity {
        model: text("SystemProductName")?,
        serial: text("SystemSerialNumber")?,
        mlb: text("MLB")?,
        uuid: text("SystemUUID")?,
        rom: rom.iter().map(|b| format!("{b:02X}")).collect(),
    })
}

fn rom_bytes(rom: &str) -> Result<Vec<u8>> {
    ensure!(
        rom.len() == 12 && rom.chars().all(|c| c.is_ascii_hexdigit()),
        "a ROM is 12 hex digits, not {rom:?}"
    );
    (0..6)
        .map(|i| u8::from_str_radix(&rom[i * 2..i * 2 + 2], 16).map_err(Into::into))
        .collect()
}

/// `config.plist` presenting `id`; everything else in it stays as it was.
pub fn patch_config(bytes: &[u8], id: &Identity) -> Result<Vec<u8>> {
    let mut root = Value::from_reader(std::io::Cursor::new(bytes)).context("read config.plist")?;
    let g = root
        .as_dictionary_mut()
        .and_then(|d| d.get_mut("PlatformInfo"))
        .and_then(Value::as_dictionary_mut)
        .and_then(|d| d.get_mut("Generic"))
        .and_then(Value::as_dictionary_mut)
        .context("config.plist has no PlatformInfo > Generic")?;
    g.insert("SystemProductName".into(), Value::String(id.model.clone()));
    g.insert(
        "SystemSerialNumber".into(),
        Value::String(id.serial.clone()),
    );
    g.insert("MLB".into(), Value::String(id.mlb.clone()));
    g.insert("SystemUUID".into(), Value::String(id.uuid.clone()));
    g.insert("ROM".into(), Value::Data(rom_bytes(&id.rom)?));
    let mut out = Vec::new();
    root.to_writer_xml(&mut out).context("write config.plist")?;
    Ok(out)
}

fn vm_image(dir: &Path) -> Result<(PathBuf, String)> {
    let files = crate::passthrough::read_vm_files(dir)?;
    let image = files
        .opencore
        .context("vm.toml names no OpenCore image ([q35] opencore): only x86-64 VMs have one")?;
    Ok((image, files.opencore_format))
}

/// The identity the VM in `dir` presents, read from its OpenCore image.
pub fn read_vm(dir: &Path) -> Result<Identity> {
    let (image, format) = vm_image(dir)?;
    read_config(&crate::opencore::read_config(&image, &format)?)
}

/// What [`renew_vm`] did.
#[derive(Debug, Clone)]
pub struct Renewed {
    pub old: Identity,
    pub new: Identity,
    /// The image as it was before Cuvée first changed it.
    pub backup: PathBuf,
}

/// Gives the VM `name` (in `dir`, off) a new identity: a new `model` (or the one it has), with a
/// new MAC address as its ROM. The image's config.plist gets the new values and keeps the
/// rest; reims-ctl gets the new address.
pub fn renew_vm(name: &str, dir: &Path, model: Option<Model>) -> Result<Renewed> {
    let (image, format) = vm_image(dir)?;
    let old = read_config(&crate::opencore::read_config(&image, &format)?)?;
    let model = match model {
        Some(m) => m,
        None => Model::parse(&old.model).with_context(|| {
            format!(
                "the VM presents a {}, which reims-util does not make: give --model",
                old.model
            )
        })?,
    };
    let mac = random_mac()?;
    let new = generate(model, Some(&mac))?;
    let backup =
        crate::opencore::edit_config(&image, &format, |b| patch_config(b, &new).map(Some))?
            .context("config.plist was not changed")?;
    let mut set = crate::paths::reims_ctl_command();
    set.args(["set", name, "--mac", &mac]);
    let out = set.output().context("run reims-ctl set")?;
    ensure!(
        out.status.success(),
        "the image presents the new Mac, but reims-ctl set --mac {mac} failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(Renewed { old, new, backup })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Misc</key><dict><key>Boot</key><dict><key>ShowPicker</key><true/></dict></dict>
<key>PlatformInfo</key><dict><key>Generic</key><dict>
<key>MLB</key><string>C02126700QXPHC18C</string>
<key>ROM</key><data>UlQAq83v</data>
<key>SpoofVendor</key><true/>
<key>SystemProductName</key><string>iMac20,1</string>
<key>SystemSerialNumber</key><string>C02FXCYBPN5T</string>
<key>SystemUUID</key><string>B84382F7-339D-4DE1-92A5-0CFFD05BF468</string>
</dict></dict>
</dict></plist>"#;

    fn ours() -> Identity {
        Identity {
            model: "iMacPro1,1".into(),
            serial: "C02T5ZZZHX87".into(),
            mlb: "C02051310QXPHC11F".into(),
            uuid: "00000000-1111-4222-8333-444444444444".into(),
            rom: "5254001A2B3C".into(),
        }
    }

    #[test]
    fn reads_platform_info() {
        let id = read_config(CONFIG.as_bytes()).unwrap();
        assert_eq!(id.model, "iMac20,1");
        assert_eq!(id.serial, "C02FXCYBPN5T");
        assert_eq!(id.rom, "525400ABCDEF");
        assert_eq!(id.rom_as_mac(), "52:54:00:ab:cd:ef");
    }

    #[test]
    fn patching_changes_the_identity_alone() {
        let out = patch_config(CONFIG.as_bytes(), &ours()).unwrap();
        assert_eq!(read_config(&out).unwrap(), ours());
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("<key>SpoofVendor</key>"));
        assert!(text.contains("<key>ShowPicker</key>"));
    }

    #[test]
    fn reads_reims_utils_json() {
        let out = r#"{"event":"note","message":"Wrote /tmp/x/oc.img"}
{"event":"done","result":{"image":"/tmp/x/oc.img","format":"raw","opencore":"1.0.8","smbios":{"model":"iMac20,1","serial":"C02FXCYBPN5T","mlb":"C02126700QXPHC18C","uuid":"B84382F7-339D-4DE1-92A5-0CFFD05BF468","rom":"525400ABCDEF"},"ovmf":null}}"#;
        let id = parse_reims_util(out).unwrap();
        assert_eq!(id, read_config(CONFIG.as_bytes()).unwrap());
        let err = parse_reims_util(r#"{"event":"error","message":"no network"}"#).unwrap_err();
        assert!(err.to_string().contains("no network"));
        assert!(parse_reims_util("").is_err());
    }

    #[test]
    fn a_new_mac_is_qemus_and_unicast() {
        let m = random_mac().unwrap();
        assert!(m.starts_with("52:54:00:"), "{m}");
        assert_eq!(m.len(), 17);
    }

    #[test]
    fn a_model_given_must_run_the_version() {
        assert_eq!(
            pick_model(None, Some(Version::Mojave)).unwrap(),
            Model::IMacPro1_1
        );
        assert_eq!(pick_model(None, None).unwrap(), Model::IMac20_1);
        assert!(pick_model(Some("iMacPro1,1"), Some(Version::Tahoe)).is_err());
        assert!(pick_model(Some("MacPro7,1"), None).is_err());
    }
}
