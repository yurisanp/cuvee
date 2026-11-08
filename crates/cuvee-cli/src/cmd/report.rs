//! `cuvee report [VM]`: writes the bug-report text file (cuvee_core::report) and, with a VM,
//! the zip of `reims-ctl report VM`.

use crate::ui::{self, Arg};
use anyhow::Result;
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct Args {
    /// A VM whose logs and settings go in too (through `reims-ctl report`)
    pub vm: Option<String>,
}

pub fn run(args: Args) -> Result<ExitCode> {
    let path = cuvee_core::report::write(args.vm.as_deref())?;
    let path = path.display().to_string();
    ui::info(&ui::t("cli-report-saved", &[("path", Arg::S(&path))]));
    ui::info(&ui::t("cli-report-next", &[]));
    Ok(ExitCode::SUCCESS)
}
