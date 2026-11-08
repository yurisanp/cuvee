//! Everything that is not cuvee's own goes to reims-ctl, with this install's QEMU and
//! reims-util (see cuvee_core::paths).

use anyhow::{bail, Result};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::ExitCode;

pub fn exec(args: Vec<OsString>) -> Result<ExitCode> {
    if cuvee_core::paths::reims_ctl().is_none() {
        bail!("{}", crate::ui::t("cli-reims-missing", &[]));
    }
    let err = cuvee_core::paths::reims_ctl_command().args(args).exec();
    bail!("reims-ctl: {err}")
}
