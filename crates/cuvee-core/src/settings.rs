//! The app's own settings, a small TOML file in the XDG config directory.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// A BCP 47 tag the translations know: "pt-BR", "en".
    pub language: String,
    /// No animation: set by the user, or by the desktop's reduce-motion preference.
    pub reduce_motion: bool,
    /// What new machines get beyond CPUs, memory and disk; all of it is optional.
    pub advanced: Advanced,
}

/// Advanced choices for new machines. `None` leaves the choice to reims-ctl, so a field only
/// becomes a flag once the user sets it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Advanced {
    /// The section is open in Ajustes.
    pub open: bool,
    /// 1 to 8 windows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displays: Option<u32>,
    /// `host`, or WIDTHxHEIGHT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidpi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub borderless: Option<bool>,
    /// Hz; 0 for the host screen's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<u32>,
    /// `auto`, a number or name from `reims-ctl gpus`, or a Vulkan ICD JSON file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_acceleration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    /// pipewire, pa, alsa, none or auto.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<bool>,
    /// 127.0.0.1's port forwarded to the guest's SSH; 0 for none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<bool>,
    /// REIMS_VGPU_LOG's filter, such as `info` or `warn,reims_vgpu_core=debug`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    /// Seconds `stop` waits for the guest before it powers the VM off; 0 for no limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_timeout: Option<u64>,
    /// Download a full offline installer (13 to 20 GB) instead of the recovery image.
    pub offline: bool,
    /// A folder for the machines, each in `<folder>/<name>`; none for reims-ctl's own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<PathBuf>,
    /// More QEMU arguments, as one shell-quoted string.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub extra_args: String,
    /// KEY=VALUE pairs for QEMU's environment, separated by spaces.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub env: String,
}

/// A resolution reims-ctl takes: `host`, or WIDTHxHEIGHT with sizes from 480 to 16384.
pub fn valid_resolution(s: &str) -> bool {
    if s == "host" {
        return true;
    }
    let Some((w, h)) = s.split_once('x') else {
        return false;
    };
    let size = |v: &str| {
        !v.is_empty()
            && v.chars().all(|c| c.is_ascii_digit())
            && v.parse::<u32>().is_ok_and(|n| (480..=16384).contains(&n))
    };
    size(w) && size(h)
}

/// An environment entry reims-ctl takes: KEY=VALUE, KEY of letters, digits and _, not
/// starting with a digit.
pub fn valid_env(s: &str) -> bool {
    let Some((k, _)) = s.split_once('=') else {
        return false;
    };
    !k.is_empty()
        && !k.starts_with(|c: char| c.is_ascii_digit())
        && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn flag(out: &mut Vec<String>, name: &str, value: Option<String>) {
    if let Some(v) = value {
        out.push(format!("--{name}"));
        out.push(v);
    }
}

impl Advanced {
    /// The flags `reims-ctl create` takes for these choices, for a VM named `name`.
    pub fn create_args(&self, name: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(dir) = &self.folder {
            out.push("--dir".into());
            out.push(dir.join(name).to_string_lossy().into_owned());
        }
        if self.offline {
            out.push("--offline".into());
        }
        flag(
            &mut out,
            "displays",
            self.displays.map(|n| n.clamp(1, 8).to_string()),
        );
        flag(
            &mut out,
            "resolution",
            self.resolution.clone().filter(|r| valid_resolution(r)),
        );
        flag(&mut out, "hidpi", self.hidpi.map(|b| b.to_string()));
        flag(
            &mut out,
            "borderless",
            self.borderless.map(|b| b.to_string()),
        );
        flag(&mut out, "refresh", self.refresh.map(|n| n.to_string()));
        flag(
            &mut out,
            "gpu",
            self.gpu.clone().filter(|g| !g.trim().is_empty()),
        );
        flag(&mut out, "logs", self.logs.map(|b| b.to_string()));
        flag(&mut out, "audio", self.audio.map(|b| b.to_string()));
        flag(&mut out, "ssh-port", self.ssh_port.map(|n| n.to_string()));
        out
    }

    /// The flags only `reims-ctl set` takes; empty when there are none to give.
    pub fn set_args(&self) -> Vec<String> {
        let mut out = Vec::new();
        flag(
            &mut out,
            "video-acceleration",
            self.video_acceleration.map(|b| b.to_string()),
        );
        flag(&mut out, "audio-backend", self.audio_backend.clone());
        flag(&mut out, "network", self.network.map(|b| b.to_string()));
        flag(
            &mut out,
            "log-level",
            self.log_level.clone().filter(|l| !l.trim().is_empty()),
        );
        flag(
            &mut out,
            "stop-timeout",
            self.stop_timeout.map(|n| n.to_string()),
        );
        if !self.extra_args.trim().is_empty() {
            out.push("--extra-args".into());
            out.push(self.extra_args.trim().to_string());
        }
        for e in self.env_entries() {
            out.push("--env".into());
            out.push(e);
        }
        out
    }

    /// The valid KEY=VALUE entries of [`Advanced::env`].
    pub fn env_entries(&self) -> Vec<String> {
        self.env
            .split_whitespace()
            .filter(|l| valid_env(l))
            .map(String::from)
            .collect()
    }

    /// Back to reims-ctl's own choices, keeping the section open.
    pub fn reset(&mut self) {
        *self = Self {
            open: self.open,
            ..Self::default()
        };
    }

    /// Whether any choice differs from reims-ctl's.
    pub fn is_default(&self) -> bool {
        let mut d = self.clone();
        d.open = false;
        d == Self::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: crate::i18n::default_language().to_string(),
            reduce_motion: false,
            advanced: Advanced::default(),
        }
    }
}

pub fn path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| Path::new(v).is_absolute())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        });
    base.join("cuvee/settings.toml")
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(path())
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let p = path();
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        std::fs::write(&p, toml::to_string_pretty(self)?)
            .with_context(|| format!("writing {}", p.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_set_gives_no_flags() {
        let a = Advanced::default();
        assert!(a.create_args("mac").is_empty());
        assert!(a.set_args().is_empty());
        assert!(a.is_default());
    }

    #[test]
    fn choices_become_flags() {
        let a = Advanced {
            open: true,
            displays: Some(2),
            resolution: Some("2560x1440".into()),
            hidpi: Some(true),
            refresh: Some(144),
            audio: Some(false),
            ssh_port: Some(2222),
            offline: true,
            folder: Some(PathBuf::from("/mnt/dados/vms")),
            video_acceleration: Some(false),
            audio_backend: Some("pipewire".into()),
            stop_timeout: Some(30),
            extra_args: " -device usb-tablet ".into(),
            env: "MESA_DEBUG=1 1BAD=x notanentry".into(),
            ..Default::default()
        };
        assert_eq!(
            a.create_args("mac").join(" "),
            "--dir /mnt/dados/vms/mac --offline --displays 2 --resolution 2560x1440 \
             --hidpi true --refresh 144 --audio false --ssh-port 2222"
        );
        assert_eq!(
            a.set_args(),
            [
                "--video-acceleration",
                "false",
                "--audio-backend",
                "pipewire",
                "--stop-timeout",
                "30",
                "--extra-args",
                "-device usb-tablet",
                "--env",
                "MESA_DEBUG=1"
            ]
        );
        assert!(!a.is_default());
    }

    #[test]
    fn bad_resolution_is_left_out() {
        assert!(valid_resolution("host"));
        assert!(valid_resolution("1920x1080"));
        assert!(!valid_resolution("1920x"));
        assert!(!valid_resolution("10x10"));
        assert!(!valid_resolution("abc"));
        let a = Advanced {
            resolution: Some("19x".into()),
            ..Default::default()
        };
        assert!(a.create_args("mac").is_empty());
    }

    #[test]
    fn settings_round_trip_through_toml() {
        let mut s = Settings::default();
        s.advanced.displays = Some(3);
        s.advanced.extra_args = "-smp sockets=1".into();
        let text = toml::to_string_pretty(&s).unwrap();
        let back: Settings = toml::from_str(&text).unwrap();
        assert_eq!(back, s);
        // An older file, without the section, still loads.
        let old: Settings = toml::from_str("language = \"en\"\nreduce_motion = true\n").unwrap();
        assert!(old.advanced.is_default());
    }
}
