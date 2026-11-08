//! `cuvee doctor`: cuvee_core::host's checks, translated, then reims-ctl's own.

use crate::ui::{self, Arg};
use anyhow::Result;
use cuvee_core::host::{self, GpuVendor, Level};
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct Args {
    /// Skip `reims-ctl doctor` after Cuvée's own checks
    #[arg(long)]
    pub no_reims: bool,
}

pub fn run(args: Args) -> Result<ExitCode> {
    let report = host::inspect();
    ui::title(&ui::t("cli-doctor-title", &[]));
    let distro = cuvee_core::distro::Distro::detect();
    ui::info(&ui::t("cli-distro", &[("name", Arg::S(&distro.name))]));
    let gpus: Vec<&str> = report
        .gpus
        .iter()
        .map(|g| match g {
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Other => "?",
        })
        .collect();
    ui::info(&ui::t(
        "cli-gpus",
        &[(
            "list",
            Arg::S(&if gpus.is_empty() {
                "—".into()
            } else {
                gpus.join(", ")
            }),
        )],
    ));
    println!();
    let (mut problems, mut warnings) = (0, 0);
    for c in &report.checks {
        let title = ui::t(c.id.title_key(), &[]);
        let line = match c.advice_key {
            Some(k) => format!("{title}: {}", ui::t(k, &[])),
            None => title,
        };
        match c.level {
            Level::Ok => ui::ok(&line),
            Level::Warn => {
                warnings += 1;
                ui::warn(&line)
            }
            Level::Bad => {
                problems += 1;
                ui::bad(&line)
            }
        }
        if c.level != Level::Ok {
            if let Some(fix) = &c.fix_command {
                ui::info(&format!(
                    "    {}",
                    ui::t("cli-doctor-fix", &[("command", Arg::S(fix))])
                ));
            }
        }
    }
    println!();
    let w = Arg::N(warnings as f64);
    if problems > 0 {
        println!(
            "{}",
            ui::red(&ui::t(
                "cli-doctor-summary-bad",
                &[("problems", Arg::N(problems as f64)), ("warnings", w)]
            ))
        );
    } else {
        println!(
            "{}",
            ui::green(&ui::t("cli-doctor-summary-ok", &[("warnings", w)]))
        );
    }

    if !args.no_reims && report.reims_ctl.is_some() && report.qemu.is_some() {
        ui::title(&ui::t("cli-doctor-reims", &[]));
        // Its exit code is not ours: reims-ctl reports "no install record" for any install
        // that did not come from its own install script, which is every Cuvée install.
        if let Ok(out) = cuvee_core::paths::reims_ctl_command()
            .arg("doctor")
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).into_owned()
                + &String::from_utf8_lossy(&out.stderr);
            for l in text.lines().filter(|l| !l.contains("no install record")) {
                println!("{l}");
            }
        }
    }
    Ok(if problems == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
