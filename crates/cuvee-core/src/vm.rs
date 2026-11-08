//! The VMs reims-ctl manages, read with `reims-ctl list --json`.
//!
//! The JSON is a list of `{"name", "dir", "state", "pid"}` objects. `reims-ctl show NAME`
//! has the rest of a VM's settings; the GUI reads those on demand.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Vm {
    pub name: String,
    pub dir: PathBuf,
    pub state: State,
    pub pid: Option<u32>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    dir: PathBuf,
    state: String,
    pid: Option<u32>,
}

/// Parses `reims-ctl list --json`'s output.
pub fn parse_list(json: &str) -> Result<Vec<Vm>> {
    let entries: Vec<Entry> = serde_json::from_str(json).context("reims-ctl list --json")?;
    Ok(entries
        .into_iter()
        .map(|e| Vm {
            name: e.name,
            dir: e.dir,
            state: match e.state.as_str() {
                "running" => State::Running,
                "stopped" => State::Stopped,
                _ => State::Unknown,
            },
            pid: e.pid,
        })
        .collect())
}

/// Runs `reims-ctl list --json`. An empty list when reims-ctl is not installed.
pub fn list() -> Result<Vec<Vm>> {
    let out = match crate::paths::reims_ctl_command()
        .args(["list", "--json"])
        .output()
    {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).context("running reims-ctl"),
    };
    if !out.status.success() {
        anyhow::bail!(
            "reims-ctl list failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    parse_list(&String::from_utf8_lossy(&out.stdout))
}

/// `reims-ctl start NAME`, `stop NAME`, `stop --force NAME`: the command to run, not run here,
/// so the GUI can show it and run it in the background.
pub fn command(action: Action, name: &str) -> Command {
    let mut c = crate::paths::reims_ctl_command();
    match action {
        Action::Start => c.arg("start"),
        Action::Stop => c.arg("stop"),
        Action::PowerOff => c.args(["stop", "--force"]),
    };
    c.arg(name);
    c
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    PowerOff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reims_ctl_list() {
        let json = r#"[{"name":"mac","dir":"/home/u/.local/share/reims-ctl/vms/mac","state":"running","pid":4242},
                       {"name":"old","dir":"/vms/old","state":"stopped","pid":null}]"#;
        let vms = parse_list(json).unwrap();
        assert_eq!(vms.len(), 2);
        assert_eq!(vms[0].state, State::Running);
        assert_eq!(vms[0].pid, Some(4242));
        assert_eq!(vms[1].state, State::Stopped);
    }
}

/// How long the process `pid` has run, from /proc.
pub fn uptime(pid: u32) -> Option<std::time::Duration> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 22 (1-based) is starttime in clock ticks; the command name may hold spaces, so
    // count from after the closing parenthesis.
    let rest = &stat[stat.rfind(')')? + 2..];
    let start_ticks: f64 = rest.split_whitespace().nth(19)?.parse().ok()?;
    let uptime: f64 = std::fs::read_to_string("/proc/uptime")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let secs = uptime - start_ticks / clock_ticks_per_second();
    (secs > 0.0).then(|| std::time::Duration::from_secs_f64(secs))
}

fn clock_ticks_per_second() -> f64 {
    // CLOCK_TCK is 100 on every mainstream Linux build; `getconf CLK_TCK` confirms it.
    std::process::Command::new("getconf")
        .arg("CLK_TCK")
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(100.0)
}

/// One reading of a process's CPU time (ticks) and resident memory (bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Usage {
    pub cpu_ticks: u64,
    pub rss_bytes: u64,
}

pub fn usage(pid: u32) -> Option<Usage> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = &stat[stat.rfind(')')? + 2..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    let utime: u64 = f.get(11)?.parse().ok()?;
    let stime: u64 = f.get(12)?.parse().ok()?;
    let rss_pages: u64 = f.get(21)?.parse().ok()?;
    Some(Usage {
        cpu_ticks: utime + stime,
        rss_bytes: rss_pages * 4096,
    })
}

/// CPU share (0–100 over all host cores) between two readings `seconds` apart.
pub fn cpu_percent(before: Usage, after: Usage, seconds: f32, host_cpus: u32) -> f32 {
    if seconds <= 0.0 || host_cpus == 0 {
        return 0.0;
    }
    let ticks = after.cpu_ticks.saturating_sub(before.cpu_ticks) as f32;
    (ticks / clock_ticks_per_second() as f32 / seconds / host_cpus as f32 * 100.0).clamp(0.0, 100.0)
}

pub fn host_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

pub fn host_memory_gib() -> u32 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<u64>().ok())
        })
        .map(|kb| ((kb as f64) / 1024.0 / 1024.0).round() as u32)
        .unwrap_or(0)
}

/// The `reims-ctl create` command for a new x86-64 VM; shown in the review step and run as is.
/// With `identity`, the VM gets that staged OpenCore image (a copy) and its ROM as its network
/// card's address; without, reims-ctl makes it an image with a new identity.
pub fn create_command(
    name: &str,
    macos_slug: &str,
    cpus: u32,
    memory_gib: u32,
    disk_gib: u32,
    start: bool,
    identity: Option<&crate::smbios::Staged>,
) -> Command {
    let mut c = crate::paths::reims_ctl_command();
    c.arg("create")
        .arg(name)
        .args(["--macos", macos_slug])
        .args(["--cpus", &cpus.to_string()])
        .args(["--memory", &format!("{memory_gib}G")])
        .args(["--disk-size", &format!("{disk_gib}G")]);
    if let Some(s) = identity {
        c.arg("--opencore").arg(&s.image).args(["--mac", &s.mac]);
    }
    if start {
        c.arg("--start");
    }
    c
}

/// A command as the user would type it.
pub fn display(c: &Command) -> String {
    // reims-ctl by name, not by the user runtime's full path.
    let program = std::path::Path::new(c.get_program());
    let program = match program.file_name() {
        Some(n) if n == "reims-ctl" => n,
        _ => c.get_program(),
    };
    std::iter::once(program)
        .chain(c.get_args())
        .map(|a| a.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}
