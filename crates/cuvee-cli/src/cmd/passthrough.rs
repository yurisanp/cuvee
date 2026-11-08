//! `cuvee passthrough`: hand a physical GPU to a macOS 10.13–15 VM (VFIO).
//!
//! Everything it decides comes from `cuvee_core::passthrough`; this file only parses the
//! command line, prints, and runs what needs root through sudo.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{bail, Context, Result};
use cuvee_core::i18n::{self, Locale};
use cuvee_core::macos::Version;
use cuvee_core::passthrough::{self as pt, oclp, Assessment, Finding, Gpu, Level};
use cuvee_core::vm;
use std::path::Path;
use std::process::{Command, ExitCode};

#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand, Debug)]
enum Cmd {
    /// List the host's GPUs and whether each can go to a VM of this macOS
    List {
        /// macOS version: sequoia, 15, 10.13...
        #[arg(long, default_value = "sequoia")]
        macos: String,
        /// The VM's RAM in GiB, for the locked-memory check
        #[arg(long, default_value_t = 8)]
        memory: u32,
    },
    /// Everything that stops GPU from going to a VM of this macOS (exit 1 if anything does)
    Check {
        /// The GPU's PCI address, as list shows it: 0c:00 or 0000:0c:00.0
        gpu: String,
        #[arg(long, default_value = "sequoia")]
        macos: String,
        #[arg(long, default_value_t = 8)]
        memory: u32,
    },
    /// Set the host up for GPU: vfio-pci at boot, udev rule, locked-memory limit (uses sudo)
    SetupHost {
        gpu: String,
        /// Also move the GPU to vfio-pci now, without a reboot
        #[arg(long)]
        now: bool,
        /// Print the script instead of running it
        #[arg(long)]
        print: bool,
    },
    /// Remove what setup-host wrote (uses sudo)
    UndoHost {
        #[arg(long)]
        print: bool,
    },
    /// Give GPU to VM from its next start
    Attach { vm: String, gpu: String },
    /// Take the GPU back out of VM
    Detach { vm: String },
    /// Set VM's OpenCore up for OpenCore Legacy Patcher's root patches (Kepler, GCN 1–3 on 13–15)
    Oclp {
        vm: String,
        #[arg(long)]
        macos: String,
    },
}

fn locale() -> Locale {
    Locale::new(i18n::system_language().unwrap_or(i18n::default_language()))
}

fn version(s: &str) -> Result<Version> {
    Version::parse(s).with_context(|| format!("unknown macOS version {s:?}"))
}

fn mark(l: Level) -> &'static str {
    match l {
        Level::Ok => "✓",
        Level::Warn => "!",
        Level::Bad => "✗",
    }
}

fn detail(f: &Finding) -> Option<String> {
    match f {
        Finding::IommuOff { missing_params } if !missing_params.is_empty() => {
            Some(missing_params.join(" "))
        }
        Finding::GroupShared(d) | Finding::NotBound(d) => Some(d.join(", ")),
        Finding::VfioAccess(g) => Some(pt::pci::vfio_node(*g).display().to_string()),
        Finding::Memlock { have, need } => Some(format!("{} MiB < {} MiB", have >> 20, need >> 20)),
        _ => None,
    }
}

fn verdict(l: &Locale, a: &Assessment) -> String {
    if a.ready() {
        l.t("pt-ready")
    } else if a.possible() {
        l.t("pt-possible")
    } else {
        l.t("pt-blocked")
    }
}

fn header(l: &Locale, g: &Gpu, a: &Assessment) {
    let p = &g.primary;
    println!(
        "{} {}  {}  [{}]  {}, {}",
        mark(a.worst()),
        p.address,
        l.t(g.family.label_key()),
        p.id(),
        l.t(a.support.label_key()),
        match p.iommu_group {
            Some(n) => l.t_args("pt-group", &[("group", Locale::num(n))]),
            None => "-".into(),
        }
    );
    println!(
        "    {}",
        l.t_args(
            "pt-driver",
            &[("driver", Locale::str(p.driver.as_deref().unwrap_or("-")))]
        )
    );
}

fn print_findings(l: &Locale, a: &Assessment) {
    for f in &a.findings {
        println!("    {} {}", mark(f.level()), l.t(f.key()));
        if let Some(d) = detail(f) {
            println!("      {d}");
        }
    }
    println!("    → {}", verdict(l, a));
}

fn pick(gpus: &[(Gpu, Assessment)], address: &str, l: &Locale) -> Result<usize> {
    let only: Vec<Gpu> = gpus.iter().map(|(g, _)| g.clone()).collect();
    let g = pt::find(&only, address)
        .with_context(|| l.t_args("pt-gpu-missing", &[("gpu", Locale::str(address))]))?;
    Ok(gpus.iter().position(|(x, _)| x == g).unwrap())
}

fn sudo_sh(script: &str) -> Result<()> {
    let st = Command::new("sudo")
        .args(["sh", "-c", script])
        .status()
        .context("run sudo")?;
    if !st.success() {
        bail!("the script failed ({st})");
    }
    Ok(())
}

fn find_vm(name: &str, l: &Locale) -> Result<vm::Vm> {
    vm::list()?
        .into_iter()
        .find(|v| v.name == name)
        .with_context(|| l.t_args("pt-vm-missing", &[("vm", Locale::str(name))]))
}

fn run_cmd(c: &mut Command) -> Result<()> {
    let st = c
        .status()
        .with_context(|| format!("run {}", vm::display(c)))?;
    if !st.success() {
        bail!("{} failed ({st})", vm::display(c));
    }
    Ok(())
}

pub fn run_args(args: Args) -> Result<ExitCode> {
    let l = locale();
    match args.cmd {
        Cmd::List { macos, memory } => {
            let (host, gpus) = pt::survey(version(&macos)?, memory);
            if gpus.is_empty() {
                println!("{}", l.t("pt-none"));
            }
            for (g, a) in &gpus {
                header(&l, g, a);
                print_findings(&l, a);
            }
            let params = pt::missing_kernel_params(host.cpu, &host.cmdline);
            if !params.is_empty() {
                println!(
                    "\n{}",
                    l.t_args(
                        "pt-kernel-params",
                        &[("params", Locale::str(&params.join(" ")))]
                    )
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Check { gpu, macos, memory } => {
            let (_, gpus) = pt::survey(version(&macos)?, memory);
            let (g, a) = &gpus[pick(&gpus, &gpu, &l)?];
            header(&l, g, a);
            print_findings(&l, a);
            Ok(if a.ready() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Cmd::SetupHost { gpu, now, print } => {
            let (host, gpus) = pt::survey(Version::Sequoia, 8);
            let (g, a) = &gpus[pick(&gpus, &gpu, &l)?];
            if a.findings.contains(&Finding::HostScreen { only: true }) {
                bail!(l.t("pt-find-host-screen-only"));
            }
            let plan = pt::host_plan(g, pt::initramfs(), host.cpu, &host.cmdline);
            let script = plan.script(now);
            if print {
                print!("{script}");
                return Ok(ExitCode::SUCCESS);
            }
            sudo_sh(&script)?;
            println!(
                "{}",
                l.t(if now {
                    "pt-setup-bound"
                } else {
                    "pt-setup-done"
                })
            );
            if !plan.kernel_params.is_empty() {
                println!(
                    "{}",
                    l.t_args(
                        "pt-kernel-params",
                        &[("params", Locale::str(&plan.kernel_params.join(" ")))]
                    )
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::UndoHost { print } => {
            let script = pt::undo_script(pt::initramfs());
            if print {
                print!("{script}");
            } else {
                sudo_sh(&script)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Attach { vm: name, gpu } => {
            let v = find_vm(&name, &l)?;
            let files = pt::read_vm_files(&v.dir)?;
            let devices = pt::pci::scan(Path::new(pt::SYSFS));
            let all = pt::gpus(&devices);
            let g = pt::find(&all, &gpu)
                .with_context(|| l.t_args("pt-gpu-missing", &[("gpu", Locale::str(&gpu))]))?;
            run_cmd(&mut pt::attach_command(&name, g, &files.extra_args))?;
            println!(
                "{}",
                l.t_args(
                    "pt-attach-done",
                    &[
                        ("vm", Locale::str(&name)),
                        ("gpu", Locale::str(&g.primary.address))
                    ]
                )
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Detach { vm: name } => {
            let v = find_vm(&name, &l)?;
            let files = pt::read_vm_files(&v.dir)?;
            run_cmd(&mut pt::detach_command(&name, &files.extra_args))?;
            println!(
                "{}",
                l.t_args("pt-detach-done", &[("vm", Locale::str(&name))])
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Oclp { vm: name, macos } => {
            let ver = version(&macos)?;
            if oclp::boot_args(ver).is_empty() {
                println!("{}", l.t("pt-oclp-not-needed"));
                return Ok(ExitCode::SUCCESS);
            }
            let v = find_vm(&name, &l)?;
            if v.state == vm::State::Running {
                bail!(l.t_args("pt-vm-running", &[("vm", Locale::str(&name))]));
            }
            let files = pt::read_vm_files(&v.dir)?;
            let image = files
                .opencore
                .context("vm.toml names no OpenCore image ([q35] opencore)")?;
            match oclp::prepare(&image, &files.opencore_format, ver)? {
                oclp::Prepared::AlreadyDone => {
                    println!(
                        "{}",
                        l.t_args("pt-oclp-already", &[("vm", Locale::str(&name))])
                    )
                }
                oclp::Prepared::Patched { backup } => println!(
                    "{}",
                    l.t_args(
                        "pt-oclp-done",
                        &[
                            ("vm", Locale::str(&name)),
                            ("backup", Locale::str(&backup.display().to_string()))
                        ]
                    )
                ),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// The shape the v0.2 CLI's other subcommands have.
pub fn run(args: Args) -> Result<ExitCode> {
    run_args(args)
}
