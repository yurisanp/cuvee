//! `cuvee smbios`: the Mac a VM presents (model, serial, MLB, UUID, ROM).
//!
//! reims-ctl gives each VM it creates an identity of its own; reims-util makes the values. This
//! shows a VM's, gives a VM a new one, or prints one with no VM, as GenSMBIOS does.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

use crate::ui::{self, Arg};
use anyhow::{bail, Context, Result};
use cuvee_core::macos::Version;
use cuvee_core::smbios::{self, Identity};
use cuvee_core::vm;
use std::io::{BufRead, IsTerminal, Write};
use std::process::ExitCode;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand, Debug)]
enum Cmd {
    /// Show the Mac a VM presents
    Show {
        vm: String,
        #[arg(long)]
        json: bool,
    },
    /// Give a VM (shut down) a new serial, MLB, UUID and ROM, and a new MAC address to match
    New {
        vm: String,
        /// Present another Mac: iMac20,1 (macOS 10.15.6 to 26) or iMacPro1,1 (10.13 to 15)
        #[arg(long)]
        model: Option<String>,
        /// Don't ask
        #[arg(long, short)]
        yes: bool,
    },
    /// Print a new identity, for no VM
    Generate {
        /// The macOS it is for, which picks the model: tahoe, 15, 10.13...
        #[arg(long)]
        macos: Option<String>,
        /// iMac20,1 or iMacPro1,1
        #[arg(long)]
        model: Option<String>,
        /// The ROM, as a MAC address (default: a new one)
        #[arg(long)]
        rom: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

fn print(id: &Identity) {
    ui::info(&format!("{:<8} {}", ui::t("smbios-model", &[]), id.model));
    ui::info(&format!("{:<8} {}", ui::t("smbios-serial", &[]), id.serial));
    ui::info(&format!("{:<8} {}", "MLB", id.mlb));
    ui::info(&format!("{:<8} {}", "UUID", id.uuid));
    ui::info(&format!("{:<8} {}", "ROM", id.rom));
}

fn find_vm(name: &str) -> Result<vm::Vm> {
    vm::list()?
        .into_iter()
        .find(|v| v.name == name)
        .with_context(|| ui::t("pt-vm-missing", &[("vm", Arg::S(name))]))
}

fn confirm(question: &str) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        bail!(ui::t("smbios-new-needs-yes", &[]));
    }
    print!("{question} [s/N] ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    let a = line.trim().to_lowercase();
    Ok(matches!(a.as_str(), "s" | "sim" | "y" | "yes"))
}

pub fn run(args: Args) -> Result<ExitCode> {
    match args.cmd {
        Cmd::Show { vm: name, json } => {
            let v = find_vm(&name)?;
            let id = smbios::read_vm(&v.dir)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&id)?);
            } else {
                ui::title(&ui::t("smbios-show-title", &[("vm", Arg::S(&name))]));
                print(&id);
            }
        }
        Cmd::New {
            vm: name,
            model,
            yes,
        } => {
            let v = find_vm(&name)?;
            if v.state == vm::State::Running {
                bail!(ui::t("pt-vm-running", &[("vm", Arg::S(&name))]));
            }
            let model = model
                .as_deref()
                .map(|m| smbios::pick_model(Some(m), None))
                .transpose()?;
            ui::warn(&ui::t("smbios-new-warning", &[]));
            if !yes && !confirm(&ui::t("smbios-new-confirm", &[("vm", Arg::S(&name))]))? {
                return Ok(ExitCode::FAILURE);
            }
            let r = smbios::renew_vm(&name, &v.dir, model)?;
            ui::title(&ui::t("smbios-new-done", &[("vm", Arg::S(&name))]));
            print(&r.new);
            ui::info("");
            ui::info(&ui::t(
                "smbios-new-old",
                &[
                    ("serial", Arg::S(&r.old.serial)),
                    ("backup", Arg::S(&r.backup.display().to_string())),
                ],
            ));
            ui::info(&ui::t(
                "smbios-check-coverage",
                &[("url", Arg::S(smbios::CHECK_COVERAGE_URL))],
            ));
        }
        Cmd::Generate {
            macos,
            model,
            rom,
            json,
        } => {
            let v = macos
                .as_deref()
                .map(|s| Version::parse(s).with_context(|| format!("unknown macOS version {s:?}")))
                .transpose()?;
            let m = smbios::pick_model(model.as_deref(), v)?;
            let id = smbios::generate(m, rom.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&id)?);
            } else {
                print(&id);
                ui::info("");
                ui::info(&ui::t(
                    "smbios-check-coverage",
                    &[("url", Arg::S(smbios::CHECK_COVERAGE_URL))],
                ));
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
