//! Running commands, as root through sudo when needed.

use crate::ui::{self, Arg};
use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

/// The effective user id, from /proc (no libc bindings).
pub fn euid() -> u32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(2))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(u32::MAX)
}

pub fn user() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| {
            let out = Command::new("id").arg("-un").output().ok()?;
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
        .unwrap_or_default()
}

/// `argv` as root: directly when we are root, else through sudo.
pub fn root(argv: &[String]) -> Command {
    if euid() == 0 {
        let mut c = Command::new(&argv[0]);
        c.args(&argv[1..]);
        c
    } else {
        let mut c = Command::new("sudo");
        c.args(argv);
        c
    }
}

/// Runs `argv` as root, showing it first; errors when it fails.
pub fn run_root(argv: &[String]) -> Result<()> {
    let shown = argv.join(" ");
    if euid() != 0 {
        ui::info(&ui::t("cli-sudo", &[("command", Arg::S(&shown))]));
    }
    let status = root(argv)
        .status()
        .with_context(|| format!("running {shown}"))?;
    if !status.success() {
        bail!("{shown}: {status}");
    }
    Ok(())
}

/// Writes `content` to `path` as root (`sudo tee`).
pub fn write_root(path: &str, content: &str) -> Result<()> {
    if euid() == 0 {
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        return std::fs::write(path, content).with_context(|| format!("writing {path}"));
    }
    ui::info(&ui::t(
        "cli-sudo",
        &[("command", Arg::S(&format!("tee {path}")))],
    ));
    let mut child = Command::new("sudo")
        .args(["tee", path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .context("running sudo tee")?;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(content.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        bail!("sudo tee {path}: {status}");
    }
    Ok(())
}

/// Runs a command and says whether it worked, without showing its output.
pub fn quiet(cmd: &mut Command) -> bool {
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The groups `user` is in.
pub fn groups(user: &str) -> Vec<String> {
    Command::new("id")
        .args(["-nG", user])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether the system has a group called `name`.
pub fn group_exists(name: &str) -> bool {
    quiet(Command::new("getent").args(["group", name]))
        || std::fs::read_to_string("/etc/group")
            .map(|g| g.lines().any(|l| l.split(':').next() == Some(name)))
            .unwrap_or(false)
}
