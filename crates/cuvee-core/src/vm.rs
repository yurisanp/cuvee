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

/// The settings of a VM that Cuvée edits, read from its `vm.toml` (reims-ctl's own file).
/// They are changed through `reims-ctl set` ([`set_command`]), never by writing the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub cpus: u32,
    pub memory_mib: u64,
    /// Display ports, each with a window of its own (1 to 8).
    pub displays: u32,
    /// `WIDTHxHEIGHT`, or `host` for the host screen's modes.
    pub resolution: String,
    pub hidpi: bool,
    pub borderless: bool,
    pub audio: bool,
    pub network: bool,
    /// Host port on 127.0.0.1 forwarded to the guest's SSH; 0 for none.
    pub ssh_port: u16,
    /// Seconds `stop` waits for the guest before it powers the VM off; 0 for no limit.
    pub stop_timeout: u64,
    /// "reims-vgpu" or "vmware-svga" (a passed GPU or no acceleration).
    pub graphics: String,
}

impl Config {
    pub fn memory_gib(&self) -> u32 {
        self.memory_mib.div_ceil(1024) as u32
    }
}

/// Reads `<dir>/vm.toml`.
pub fn read_config(dir: &std::path::Path) -> Result<Config> {
    let file = dir.join("vm.toml");
    let text =
        std::fs::read_to_string(&file).with_context(|| format!("read {}", file.display()))?;
    parse_config(&text)
}

/// Parses a `vm.toml`, with reims-ctl's defaults for what it leaves out.
pub fn parse_config(text: &str) -> Result<Config> {
    let v: toml::Table = toml::from_str(text).context("parse vm.toml")?;
    let table = |k: &str| v.get(k).and_then(|t| t.as_table());
    let int =
        |t: Option<&toml::Table>, k: &str| t.and_then(|t| t.get(k)).and_then(|x| x.as_integer());
    let flag = |t: Option<&toml::Table>, k: &str, d: bool| {
        t.and_then(|t| t.get(k))
            .and_then(|x| x.as_bool())
            .unwrap_or(d)
    };
    let display = table("display");
    let audio = table("audio");
    let network = table("network");
    Ok(Config {
        cpus: int(Some(&v), "cpus").context("vm.toml has no cpus")? as u32,
        memory_mib: int(Some(&v), "memory_mib").context("vm.toml has no memory_mib")? as u64,
        displays: int(display, "count").unwrap_or(1) as u32,
        resolution: display
            .and_then(|t| t.get("resolution"))
            .and_then(|x| x.as_str())
            .unwrap_or("1920x1080")
            .to_string(),
        hidpi: flag(display, "hidpi", false),
        borderless: flag(display, "borderless", false),
        audio: flag(audio, "enabled", true),
        network: flag(network, "enabled", true),
        ssh_port: int(network, "ssh_port").unwrap_or(0) as u16,
        stop_timeout: int(Some(&v), "stop_timeout_secs").unwrap_or(90) as u64,
        graphics: v
            .get("graphics")
            .and_then(|x| x.as_str())
            .unwrap_or("reims-vgpu")
            .to_string(),
    })
}

/// Whether reims-ctl takes `r` as a resolution: `host`, or `WIDTHxHEIGHT`.
pub fn valid_resolution(r: &str) -> bool {
    r == "host"
        || r.split_once('x').is_some_and(|(w, h)| {
            w.parse::<u32>().is_ok_and(|w| w >= 640) && h.parse::<u32>().is_ok_and(|h| h >= 480)
        })
}

/// `reims-ctl set NAME …` with only what differs between `old` and `new`; None when nothing does.
pub fn set_command(name: &str, old: &Config, new: &Config) -> Option<Command> {
    let mut args: Vec<String> = Vec::new();
    let mut opt = |flag: &str, changed: bool, value: String| {
        if changed {
            args.push(format!("--{flag}"));
            args.push(value);
        }
    };
    opt("cpus", old.cpus != new.cpus, new.cpus.to_string());
    opt(
        "memory",
        old.memory_mib != new.memory_mib,
        format!("{}M", new.memory_mib),
    );
    opt(
        "displays",
        old.displays != new.displays,
        new.displays.to_string(),
    );
    opt(
        "resolution",
        old.resolution != new.resolution,
        new.resolution.clone(),
    );
    opt("hidpi", old.hidpi != new.hidpi, new.hidpi.to_string());
    opt(
        "borderless",
        old.borderless != new.borderless,
        new.borderless.to_string(),
    );
    opt("audio", old.audio != new.audio, new.audio.to_string());
    opt(
        "network",
        old.network != new.network,
        new.network.to_string(),
    );
    opt(
        "ssh-port",
        old.ssh_port != new.ssh_port,
        new.ssh_port.to_string(),
    );
    opt(
        "stop-timeout",
        old.stop_timeout != new.stop_timeout,
        new.stop_timeout.to_string(),
    );
    if args.is_empty() {
        return None;
    }
    let mut c = crate::paths::reims_ctl_command();
    c.args(["set", name]).args(args);
    Some(c)
}

/// `reims-ctl delete NAME --yes`: the VM and its directory with its disks. reims-ctl refuses a
/// running VM.
pub fn delete_command(name: &str) -> Command {
    let mut c = crate::paths::reims_ctl_command();
    c.args(["delete", name, "--yes"]);
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

    const VM_TOML: &str = r#"name = "mac"
profile = "q35"
cpus = 8
memory_mib = 16384
disk = "disk.qcow2"

[display]
count = 1
resolution = "2560x1440"
hidpi = true
borderless = false

[network]
enabled = true
mac = "52:54:00:12:34:56"
ssh_port = 2222

[q35]
opencore = "opencore.qcow2"
"#;

    #[test]
    fn reads_vm_toml() {
        let c = parse_config(VM_TOML).unwrap();
        assert_eq!(c.cpus, 8);
        assert_eq!(c.memory_gib(), 16);
        assert_eq!(c.resolution, "2560x1440");
        assert!(c.hidpi && c.audio && c.network);
        assert_eq!(c.ssh_port, 2222);
        assert_eq!(c.stop_timeout, 90);
        assert_eq!(c.graphics, "reims-vgpu");
    }

    #[test]
    fn set_passes_only_what_changed() {
        let old = parse_config(VM_TOML).unwrap();
        assert!(set_command("mac", &old, &old).is_none());
        let mut new = old.clone();
        new.cpus = 4;
        new.memory_mib = 8 * 1024;
        new.audio = false;
        new.ssh_port = 0;
        let c = set_command("mac", &old, &new).unwrap();
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "set",
                "mac",
                "--cpus",
                "4",
                "--memory",
                "8192M",
                "--audio",
                "false",
                "--ssh-port",
                "0"
            ]
        );
    }

    #[test]
    fn delete_asks_nothing() {
        let c = delete_command("mac");
        let args: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["delete", "mac", "--yes"]);
    }

    #[test]
    fn resolutions() {
        assert!(valid_resolution("host"));
        assert!(valid_resolution("1920x1080"));
        assert!(!valid_resolution("1920"));
        assert!(!valid_resolution("big"));
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
/// card's address; without, reims-ctl makes it an image with a new identity. `advanced` adds
/// the Ajustes choices `create` takes; [`advanced_set_command`] gives the rest.
#[allow(clippy::too_many_arguments)]
pub fn create_command(
    name: &str,
    macos_slug: &str,
    cpus: u32,
    memory_gib: u32,
    disk_gib: u32,
    start: bool,
    identity: Option<&crate::smbios::Staged>,
    advanced: &crate::settings::Advanced,
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
    c.args(advanced.create_args(name));
    if start {
        c.arg("--start");
    }
    c
}

/// `reims-ctl set NAME …` for the Ajustes choices `create` does not take; None when there are
/// none. Run it after `create` and before the first start.
pub fn advanced_set_command(name: &str, advanced: &crate::settings::Advanced) -> Option<Command> {
    let args = advanced.set_args();
    if args.is_empty() {
        return None;
    }
    let mut c = crate::paths::reims_ctl_command();
    c.args(["set", name]).args(args);
    Some(c)
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
        .map(|a| crate::passthrough::sh_quote(&a.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}
