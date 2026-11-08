//! `cuvee setup-host`: the host changes doctor asks for, through sudo. Each step checks first
//! and changes nothing that is already right, so running it again is safe.

use super::sys;
use crate::ui::{self, Arg};
use anyhow::{bail, Result};
use cuvee_core::host::GpuVendor;
use std::path::Path;
use std::process::ExitCode;

const MODULES_FILE: &str = "/etc/modules-load.d/cuvee.conf";
const UDEV_FILE: &str = "/etc/udev/rules.d/70-cuvee-udmabuf.rules";
const SYSCTL_FILE: &str = "/etc/sysctl.d/70-cuvee.conf";
const UDEV_RULE: &str = "# Cuvée: lets the kvm group open /dev/udmabuf.\nKERNEL==\"udmabuf\", GROUP=\"kvm\", MODE=\"0660\"\n";

#[derive(clap::Args)]
pub struct Args {}

/// What changed, so the caller knows whether a new login is needed.
pub struct Outcome {
    pub changed: bool,
    pub relogin: bool,
}

pub fn run(_args: Args) -> Result<ExitCode> {
    if sys::euid() == 0 {
        bail!("{}", ui::t("cli-setup-no-root", &[]));
    }
    let o = apply()?;
    println!();
    if o.relogin {
        println!("{}", ui::t("cli-setup-relogin", &[]));
    } else if !o.changed {
        println!("{}", ui::t("cli-setup-nothing", &[]));
    }
    println!("cuvee doctor");
    Ok(ExitCode::SUCCESS)
}

fn failed(what: &str, e: anyhow::Error) {
    ui::warn(&ui::t(
        "cli-setup-failed",
        &[("what", Arg::S(what)), ("error", Arg::S(&format!("{e:#}")))],
    ));
}

/// Applies every step; a step that fails is reported and the others still run.
pub fn apply() -> Result<Outcome> {
    ui::title(&ui::t("cli-setup-title", &[]));
    let mut o = Outcome {
        changed: false,
        relogin: false,
    };
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    if !Path::new("/dev/udmabuf").exists() {
        match sys::run_root(&s(&["modprobe", "udmabuf"])) {
            Ok(()) => {
                ui::ok(&ui::t("cli-setup-udmabuf", &[]));
                o.changed = true;
            }
            Err(e) => failed("modprobe udmabuf", e),
        }
    }
    let loads_at_boot = ["/usr/lib/modules-load.d/cuvee.conf", MODULES_FILE]
        .iter()
        .any(|p| Path::new(p).exists());
    if !loads_at_boot {
        match sys::write_root(MODULES_FILE, "# Cuvée: modules loaded at boot.\nudmabuf\n") {
            Ok(()) => {
                ui::ok(&ui::t(
                    "cli-setup-udmabuf-boot",
                    &[("path", Arg::S(MODULES_FILE))],
                ));
                o.changed = true;
            }
            Err(e) => failed(MODULES_FILE, e),
        }
    }

    if !sys::group_exists("kvm") {
        if let Err(e) = sys::run_root(&s(&["groupadd", "--system", "kvm"])) {
            failed("groupadd kvm", e);
        }
    }

    let has_rule = [
        "/usr/lib/udev/rules.d/70-cuvee-udmabuf.rules",
        "/lib/udev/rules.d/70-cuvee-udmabuf.rules",
        UDEV_FILE,
    ]
    .iter()
    .any(|p| Path::new(p).exists());
    if !has_rule {
        match sys::write_root(UDEV_FILE, UDEV_RULE) {
            Ok(()) => {
                ui::ok(&ui::t("cli-setup-udev", &[("path", Arg::S(UDEV_FILE))]));
                o.changed = true;
                let _ = sys::run_root(&s(&["udevadm", "control", "--reload"]));
                let _ = sys::run_root(&s(&[
                    "udevadm",
                    "trigger",
                    "--subsystem-match=misc",
                    "--action=change",
                ]));
            }
            Err(e) => failed(UDEV_FILE, e),
        }
    }

    let nvidia = cuvee_core::host::gpu_vendors().contains(&GpuVendor::Nvidia);
    let maps: u64 = std::fs::read_to_string("/proc/sys/vm/max_map_count")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let want = cuvee_core::host::wanted_max_map_count();
    if nvidia && maps < want {
        let r = sys::write_root(
            SYSCTL_FILE,
            &format!("# Cuvée: reims-vgpu on NVIDIA.\nvm.max_map_count = {want}\n"),
        )
        .and_then(|()| {
            sys::run_root(&s(&[
                "sysctl",
                "-q",
                "-w",
                &format!("vm.max_map_count={want}"),
            ]))
        });
        match r {
            Ok(()) => {
                ui::ok(&ui::t(
                    "cli-setup-sysctl",
                    &[
                        ("value", Arg::S(&want.to_string())),
                        ("path", Arg::S(SYSCTL_FILE)),
                    ],
                ));
                o.changed = true;
            }
            Err(e) => failed("sysctl vm.max_map_count", e),
        }
    }

    let user = sys::user();
    if !user.is_empty() && user != "root" && !sys::groups(&user).iter().any(|g| g == "kvm") {
        match sys::run_root(&s(&["usermod", "-aG", "kvm", &user])) {
            Ok(()) => {
                ui::ok(&ui::t("cli-setup-group", &[]));
                o.changed = true;
                o.relogin = true;
            }
            Err(e) => failed("usermod -aG kvm", e),
        }
    }
    Ok(o)
}
