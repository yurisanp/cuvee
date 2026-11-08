//! `cuvee`: the command line. Prepares the host, installs and updates the runtime (QEMU with
//! reims-vgpu, reims-ctl, reims-util) and hands every other command to reims-ctl.
//!
//! One file per subcommand in `cmd/`, each with `pub fn run(args) -> Result<ExitCode>`.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

mod cmd;
mod ui;

use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::process::ExitCode;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "cuvee",
    version = VERSION,
    about = "Cuvée: macOS VMs on Linux, on top of reims-ctl",
    long_about = None,
    after_help = "Any other command goes to reims-ctl (cuvee help-reims lists them).\n\
                  With no command, cuvee opens reims-ctl's interactive menu."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Checks the host: KVM, udmabuf, GPU, Vulkan, session, disk, IOMMU, install
    Doctor(cmd::doctor::Args),
    /// Fixes what doctor reports (udmabuf, udev rule, kvm group, sysctl on NVIDIA; uses sudo)
    SetupHost(cmd::setup_host::Args),
    /// Installs the dependencies, the runtime and the app, then prepares the host
    Install(cmd::install::Args),
    /// Updates cuvee, the runtime and the app to the newest release
    Update(cmd::install::UpdateArgs),
    /// Removes what cuvee install put in your home directory (VMs stay)
    Uninstall(cmd::install::UninstallArgs),
    /// GPU passthrough (VFIO) for macOS 10.13 to 15: list, check, setup-host, attach, detach
    Passthrough(cmd::passthrough::Args),
    /// The Mac a VM presents (model, serial, MLB, UUID, ROM): show, new, generate
    Smbios(cmd::smbios::Args),
    /// Writes a bug report (host, GPUs, versions, checks; a VM's logs with its name) to Downloads
    Report(cmd::report::Args),
    /// Lists every reims-ctl command
    HelpReims {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
    #[command(external_subcommand)]
    Reims(Vec<OsString>),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        None => cmd::reims::exec(Vec::new()),
        Some(Command::Doctor(a)) => cmd::doctor::run(a),
        Some(Command::SetupHost(a)) => cmd::setup_host::run(a),
        Some(Command::Install(a)) => cmd::install::run(a),
        Some(Command::Update(a)) => cmd::install::update(a),
        Some(Command::Uninstall(a)) => cmd::install::uninstall(a),
        Some(Command::Passthrough(a)) => cmd::passthrough::run(a),
        Some(Command::Report(a)) => cmd::report::run(a),
        Some(Command::Smbios(a)) => cmd::smbios::run(a),
        Some(Command::HelpReims { args }) => {
            let mut v = vec![OsString::from("help")];
            v.extend(args);
            cmd::reims::exec(v)
        }
        Some(Command::Reims(args)) => cmd::reims::exec(args),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            ui::error(&format!("{e:#}"));
            ExitCode::FAILURE
        }
    }
}
