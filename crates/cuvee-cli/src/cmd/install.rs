//! `cuvee install`, `cuvee update` and `cuvee uninstall`: the user install, in the home
//! directory, from a GitHub release (see cuvee_core::paths and cuvee_core::release).
//!
//! `install` does, in order: the distribution's packages (through sudo), the runtime
//! (QEMU, reims-ctl, reims-util), `cuvee` itself and `cuvee-gui` in ~/.local/bin, the host
//! (`setup-host`), and a `cuvee doctor` at the end.

use super::{doctor, setup_host, sys};
use crate::ui::{self, Arg};
use anyhow::{bail, Context, Result};
use cuvee_core::distro::{self, Distro, Family};
use cuvee_core::host::GpuVendor;
use cuvee_core::paths;
use cuvee_core::release::{self, Fetcher, Source};
use std::io::IsTerminal;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(clap::Args)]
pub struct Args {
    /// Answer yes to the package manager (needed without a terminal)
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Do not install the distribution's packages
    #[arg(long)]
    pub skip_deps: bool,
    /// Do not prepare the host (udmabuf, udev rule, kvm group, sysctl)
    #[arg(long)]
    pub skip_host: bool,
    /// Do not install the desktop app
    #[arg(long)]
    pub no_gui: bool,
    /// Install this release (a tag such as v0.2.0) instead of the newest
    #[arg(long, value_name = "TAG", conflicts_with = "from")]
    pub version: Option<String>,
    /// Install from a directory holding a release's files, as CI builds them
    #[arg(long, value_name = "DIR")]
    pub from: Option<PathBuf>,
    /// Unpack the runtime again even when this release's is installed
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    /// Answer yes to the package manager
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Update to this release instead of the newest
    #[arg(long, value_name = "TAG", conflicts_with = "from")]
    pub version: Option<String>,
    /// Update from a directory holding a release's files
    #[arg(long, value_name = "DIR")]
    pub from: Option<PathBuf>,
    /// Reinstall even when this release is installed
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct UninstallArgs {}

fn source(version: &Option<String>, from: &Option<PathBuf>) -> Source {
    if let Some(d) = from {
        Source::Dir(d.clone())
    } else if let Some(v) = version {
        Source::Tag(v.clone())
    } else {
        Source::from_env()
    }
}

fn refuse_if_packaged() -> Result<()> {
    if let Some(kind) = paths::packaged() {
        bail!("{}", ui::t("cli-packaged", &[("kind", Arg::S(&kind))]));
    }
    Ok(())
}

/// The release tag the user runtime in use came from (`runtime/current`'s target).
pub fn installed_tag() -> Option<String> {
    let target = std::fs::read_link(paths::runtimes_dir().join("current")).ok()?;
    Some(target.file_name()?.to_string_lossy().into_owned())
}

pub fn run(args: Args) -> Result<ExitCode> {
    refuse_if_packaged()?;
    let src = source(&args.version, &args.from);
    let tag = src.tag()?.unwrap_or_else(|| "local".into());
    println!("{}", ui::t("cli-about", &[("version", Arg::S(&tag))]));

    if !args.skip_deps {
        install_deps(args.yes)?;
    }

    let mut fetcher = Fetcher::new(src, paths::cache_dir().join("download").join(&tag));
    ui::title("Runtime");
    install_runtime(&mut fetcher, &tag, args.force)?;

    ui::title("cuvee");
    let bin = paths::user_bin_dir();
    std::fs::create_dir_all(&bin)?;
    let me = std::env::current_exe().context("finding this binary")?;
    let dest = bin.join("cuvee");
    if me.canonicalize().ok() != dest.canonicalize().ok() {
        place(&me, &dest)?;
    }
    ui::ok(&ui::t(
        "cli-installed-bin",
        &[
            ("name", Arg::S("cuvee")),
            ("path", Arg::S(&dest.display().to_string())),
        ],
    ));
    if !args.no_gui {
        install_gui(&mut fetcher, &bin)?;
    }

    if !args.skip_host {
        if sys::euid() == 0 {
            ui::warn(&ui::t("cli-setup-no-root", &[]));
        } else {
            let o = setup_host::apply()?;
            if o.relogin {
                ui::info(&ui::t("cli-setup-relogin", &[]));
            }
        }
    }

    if !on_path(&bin) {
        println!();
        ui::warn(&ui::t(
            "cli-path-missing",
            &[("dir", Arg::S(&bin.display().to_string()))],
        ));
    }
    let _ = doctor::run(doctor::Args { no_reims: false });
    println!("\n{}", ui::t("cli-install-done", &[]));
    Ok(ExitCode::SUCCESS)
}

/// The distribution's packages the runtime needs, installed through sudo when some are
/// missing. Nothing is asked when everything is there.
fn install_deps(yes: bool) -> Result<()> {
    let d = Distro::detect();
    ui::title(&ui::t("cli-distro", &[("name", Arg::S(&d.name))]));
    let gpus = cuvee_core::host::gpu_vendors();
    let wanted = distro::wanted(d.family, &gpus);
    if d.family == Family::Unknown {
        let list: Vec<&str> = wanted.iter().map(|c| *c.last().unwrap()).collect();
        ui::warn(&ui::t(
            "cli-distro-unknown",
            &[("name", Arg::S(&d.name)), ("list", Arg::S(&list.join(" ")))],
        ));
        return Ok(());
    }
    let missing = distro::missing(d.family, &wanted);
    if missing.is_empty() {
        ui::ok(&ui::t("cli-deps-ok", &[]));
    } else {
        let failed = |argv: &[String], e: anyhow::Error| {
            anyhow::anyhow!(
                "{}: {e:#}",
                ui::t("cli-deps-failed", &[("command", Arg::S(&argv.join(" ")))])
            )
        };
        // The lists first: in a fresh system they are empty, and the names are picked from them.
        if let Some(argv) = distro::refresh_command(d.family) {
            sys::run_root(&argv).map_err(|e| failed(&argv, e))?;
        }
        let pkgs = distro::resolve(d.family, &missing);
        ui::info(&ui::t(
            "cli-deps-missing",
            &[("list", Arg::S(&pkgs.join(" ")))],
        ));
        // Without a terminal the package manager cannot ask, so it is told yes.
        let yes = yes || !std::io::stdin().is_terminal();
        if let Some(argv) = distro::install_command(d.family, &pkgs, yes) {
            sys::run_root(&argv).map_err(|e| failed(&argv, e))?;
        }
        ui::ok(&ui::t("cli-deps-ok", &[]));
    }
    if gpus.contains(&GpuVendor::Nvidia) && !distro::nvidia_vulkan_installed() {
        ui::warn(&ui::t("cli-nvidia-driver", &[]));
    }
    Ok(())
}

/// Unpacks the release's runtime into runtime/<tag> and points runtime/current at it.
fn install_runtime(fetcher: &mut Fetcher, tag: &str, force: bool) -> Result<()> {
    let root = paths::runtimes_dir();
    std::fs::create_dir_all(&root)?;
    let dir = root.join(tag);
    let qemu = dir.join("qemu/bin/qemu-system-x86_64");
    if qemu.is_file() && !force {
        ui::ok(&ui::t("cli-runtime-current", &[("tag", Arg::S(tag))]));
    } else {
        ui::info(&ui::t(
            "cli-fetching",
            &[("name", Arg::S(release::RUNTIME_ASSET))],
        ));
        let archive = fetcher.fetch(release::RUNTIME_ASSET)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir).with_context(|| format!("removing {}", dir.display()))?;
        }
        release::unpack_tar_gz(&archive, &dir)?;
        let _ = std::fs::remove_file(&archive);
        // A library the distribution lacks shows here, not when the first VM starts.
        let out = Command::new(&qemu).arg("--version").output();
        let broken = match out {
            Ok(o) if o.status.success() => None,
            Ok(o) => Some(String::from_utf8_lossy(&o.stderr).trim().to_string()),
            Err(e) => Some(e.to_string()),
        };
        if let Some(err) = broken {
            let _ = std::fs::remove_dir_all(&dir);
            bail!(
                "{}",
                ui::t("cli-runtime-broken", &[("error", Arg::S(&err))])
            );
        }
        ui::ok(&ui::t(
            "cli-runtime-ready",
            &[
                ("tag", Arg::S(tag)),
                ("path", Arg::S(&dir.display().to_string())),
            ],
        ));
    }
    // Move the link in one step: a new link beside it, renamed over it.
    let link = root.join("current");
    let tmp = root.join(".current.new");
    let _ = std::fs::remove_file(&tmp);
    std::os::unix::fs::symlink(tag, &tmp)?;
    std::fs::rename(&tmp, &link)?;
    prune(&root, tag);
    Ok(())
}

/// Removes old runtimes: keeps the current one, the two newest others, and any a running
/// process executes (a VM keeps the QEMU it started with).
fn prune(root: &Path, current: &str) {
    let Ok(rd) = std::fs::read_dir(root) else {
        return;
    };
    let mut old: Vec<(std::time::SystemTime, PathBuf)> = rd
        .flatten()
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n != current && n != "current" && !n.starts_with('.') && !n.ends_with(".partial")
        })
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    old.sort_by_key(|o| std::cmp::Reverse(o.0));
    let running: Vec<PathBuf> = std::fs::read_dir("/proc")
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| std::fs::read_link(e.path().join("exe")).ok())
                .collect()
        })
        .unwrap_or_default();
    for (_, dir) in old.into_iter().skip(2) {
        if !running.iter().any(|exe| exe.starts_with(&dir)) {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

/// Copies `src` to `dest` through a temporary file, so a running `dest` is replaced, not
/// written over.
fn place(src: &Path, dest: &Path) -> Result<()> {
    let tmp = dest.with_extension("new");
    std::fs::copy(src, &tmp).with_context(|| format!("copying to {}", tmp.display()))?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    std::fs::rename(&tmp, dest).with_context(|| format!("installing {}", dest.display()))?;
    Ok(())
}

fn applications_dir() -> PathBuf {
    paths::data_dir()
        .parent()
        .map(|p| p.join("applications"))
        .unwrap_or_else(|| PathBuf::from(".local/share/applications"))
}

fn install_gui(fetcher: &mut Fetcher, bin: &Path) -> Result<()> {
    ui::title("cuvee-gui");
    ui::info(&ui::t(
        "cli-fetching",
        &[("name", Arg::S(release::GUI_ASSET))],
    ));
    let file = fetcher.fetch(release::GUI_ASSET)?;
    let dest = bin.join("cuvee-gui");
    place(&file, &dest)?;
    let _ = std::fs::remove_file(&file);
    let apps = applications_dir();
    std::fs::create_dir_all(&apps)?;
    std::fs::write(
        apps.join("cuvee.desktop"),
        format!(
            "[Desktop Entry]\nType=Application\nName=Cuvée\nGenericName=macOS virtual machines\n\
             Comment=Create and run macOS virtual machines\nExec={}\nIcon=computer\n\
             Terminal=false\nCategories=System;Emulator;\n",
            dest.display()
        ),
    )?;
    ui::ok(&ui::t(
        "cli-installed-bin",
        &[
            ("name", Arg::S("cuvee-gui")),
            ("path", Arg::S(&dest.display().to_string())),
        ],
    ));
    Ok(())
}

fn on_path(dir: &Path) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d == dir))
        .unwrap_or(false)
}

/// `cuvee update`: downloads the release's `cuvee`, puts it in ~/.local/bin and has it run
/// `install`, so the new version's own install logic does the rest.
pub fn update(args: UpdateArgs) -> Result<ExitCode> {
    refuse_if_packaged()?;
    let src = source(&args.version, &args.from);
    let tag = src.tag()?.unwrap_or_else(|| "local".into());
    let have = installed_tag();
    if have.as_deref() == Some(tag.as_str()) && !args.force {
        println!("{}", ui::t("cli-update-current", &[("tag", Arg::S(&tag))]));
        return Ok(ExitCode::SUCCESS);
    }
    println!(
        "{}",
        ui::t(
            "cli-update-to",
            &[
                ("from", Arg::S(have.as_deref().unwrap_or("—"))),
                ("to", Arg::S(&tag))
            ]
        )
    );
    let mut fetcher = Fetcher::new(src.clone(), paths::cache_dir().join("download").join(&tag));
    ui::info(&ui::t(
        "cli-fetching",
        &[("name", Arg::S(release::CLI_ASSET))],
    ));
    let new = fetcher.fetch(release::CLI_ASSET)?;
    let bin = paths::user_bin_dir();
    std::fs::create_dir_all(&bin)?;
    let dest = bin.join("cuvee");
    place(&new, &dest)?;
    let _ = std::fs::remove_file(&new);

    let mut c = Command::new(&dest);
    c.args(["install", "--skip-host"]);
    match &src {
        Source::Dir(d) => {
            c.arg("--from").arg(d);
        }
        _ => {
            c.args(["--version", &tag]);
        }
    }
    if args.yes {
        c.arg("--yes");
    }
    if args.force {
        c.arg("--force");
    }
    if !paths::user_bin_dir().join("cuvee-gui").exists() {
        c.arg("--no-gui");
    }
    let err = c.exec();
    bail!("{}: {err}", dest.display())
}

/// `cuvee uninstall`: removes what `install` put in the home directory. The VMs (reims-ctl's
/// data) and the host changes stay.
pub fn uninstall(_args: UninstallArgs) -> Result<ExitCode> {
    refuse_if_packaged()?;
    let bin = paths::user_bin_dir();
    for f in [
        bin.join("cuvee"),
        bin.join("cuvee-gui"),
        applications_dir().join("cuvee.desktop"),
    ] {
        let _ = std::fs::remove_file(f);
    }
    let _ = std::fs::remove_dir_all(paths::runtimes_dir());
    let _ = std::fs::remove_dir_all(paths::cache_dir());
    let vms = paths::data_dir()
        .parent()
        .map(|p| p.join("reims-ctl"))
        .unwrap_or_default();
    println!(
        "{}",
        ui::t(
            "cli-uninstall-done",
            &[("path", Arg::S(&vms.display().to_string()))]
        )
    );
    Ok(ExitCode::SUCCESS)
}
