//! Cuvée's desktop app: macOS virtual machines on Linux, over reims-ctl.
//!
//! SPDX-License-Identifier: GPL-3.0-or-later

mod anim;
mod screens;
mod theme;
mod widgets;

use anim::Clock;
use cuvee_core::host::{self, Report};
use cuvee_core::i18n::Locale;
use cuvee_core::macos::{Group, Version};
use cuvee_core::passthrough::{self as pt, Gpu, HostState};
use cuvee_core::settings::Settings;
use cuvee_core::smbios::{self, Staged};
use cuvee_core::vm::{self, Vm};
use iced::widget::{container, row};
use iced::{Element, Length, Subscription, Task};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(app_theme)
        .style(app_style)
        .subscription(App::subscription)
        .font(theme::FRAUNCES)
        .font(theme::INTER)
        .font(theme::JETBRAINS_MONO)
        .default_font(theme::SANS)
        .antialiasing(true)
        .window_size((1280.0, 820.0))
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Painel,
    Nova,
    Diagnostico,
    Comportamento,
    Ajustes,
    /// A running VM, by its index in `vms`.
    Execucao(usize),
    /// One VM's settings, identity and deletion: the VM in `App::maquina`.
    Maquina,
}

impl Screen {
    /// The rail highlights the section a screen belongs to.
    pub fn same_section(self, other: Screen) -> bool {
        matches!(
            (self, other),
            (
                Screen::Painel | Screen::Execucao(_) | Screen::Maquina,
                Screen::Painel | Screen::Execucao(_) | Screen::Maquina
            )
        ) || self == other
    }
}

/// The "Nova máquina" form.
#[derive(Debug, Clone)]
pub struct Nova {
    pub step: usize,
    pub version: Version,
    pub name: String,
    pub cpus: u32,
    pub memory_gib: u32,
    pub disk_gib: u32,
    /// The PCI address of the GPU to hand to the VM; None for reims-vgpu's virtual one.
    pub gpu: Option<String>,
    pub switched_at: Instant,
    pub busy: bool,
    /// The OpenCore image made for the VM, with the identity the review shows.
    pub identity: Option<Staged>,
    pub identity_making: bool,
    pub identity_error: Option<String>,
}

impl Nova {
    /// The chosen GPU, if it is still on the host.
    pub fn chosen_gpu<'a>(&self, gpus: &'a [Gpu]) -> Option<&'a Gpu> {
        self.gpu.as_deref().and_then(|a| pt::find(gpus, a))
    }

    /// The Mac the VM should present: it depends on the macOS and on a passed GPU.
    pub fn wanted_model(&self) -> cuvee_core::macos::Model {
        self.version.model_for(self.gpu.is_some())
    }

    /// The staged identity, if it is still for the Mac the VM should present.
    pub fn current_identity(&self) -> Option<&Staged> {
        self.identity
            .as_ref()
            .filter(|s| s.identity.model == self.wanted_model().name())
    }
}

#[derive(Debug, Clone)]
pub struct Exec {
    pub samples_cpu: Vec<f32>,
    pub samples_mem: Vec<f32>,
    pub last: Option<(Instant, vm::Usage)>,
    pub stopping: bool,
}

/// The "Máquina" screen: one VM's settings as read, as edited, and what is under way.
#[derive(Debug, Clone, Default)]
pub struct Maquina {
    pub name: String,
    pub dir: std::path::PathBuf,
    /// The settings in vm.toml; None until read.
    pub saved: Option<vm::Config>,
    pub edit: Option<vm::Config>,
    pub load_error: Option<String>,
    /// The Mac the VM presents, or why it could not be read.
    pub identity: Option<Result<smbios::Identity, String>>,
    pub more: bool,
    pub saving: bool,
    pub renewing: bool,
    pub confirm_renew: bool,
    pub confirm_delete: bool,
    pub deleting: bool,
}

pub struct App {
    pub locale: Locale,
    pub settings: Settings,
    pub now: Instant,
    pub screen: Screen,
    pub entered_at: Instant,
    pub report: Option<Arc<Report>>,
    pub vms: Vec<Vm>,
    /// The host's GPUs and what passthrough needs from the host, read once at start.
    pub passthrough: Option<Arc<(HostState, Vec<Gpu>)>>,
    pub host_cpus: u32,
    pub host_mem_gib: u32,
    pub nova: Nova,
    pub group: Group,
    pub group_switched_at: Instant,
    pub diag_show_cmd: bool,
    pub diag_fixing: bool,
    pub exec: Exec,
    pub maquina: Maquina,
    pub notice: Option<(Instant, String)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
    Go(Screen),
    Refresh,
    Loaded(Arc<Report>, Vec<Vm>),
    LoadFailed(String),
    PassthroughLoaded(Arc<(HostState, Vec<Gpu>)>),
    // Nova máquina
    NovaPick(Version),
    NovaName(String),
    NovaCpus(u32),
    NovaMemory(u32),
    NovaDisk(u32),
    NovaGpu(Option<String>),
    NovaNext,
    NovaBack,
    NovaCreate,
    NovaCreated(Result<(), String>),
    NovaIdentityAgain,
    NovaIdentity(Result<Staged, String>),
    // Diagnóstico
    DiagFix,
    DiagFixed(Result<(), String>),
    DiagToggleCmd,
    DiagReport,
    DiagReported(Result<String, String>),
    // Comportamento
    PickGroup(Group),
    // Ajustes
    SetLanguage(&'static str),
    SetReduceMotion(bool),
    Advanced(screens::ajustes::Edit),
    // Execução
    VmStart(String),
    VmStop(String, bool),
    VmActionDone(Result<(), String>),
    ExecSample,
    // Máquina
    MaqOpen(String),
    MaqLoaded(
        String,
        Result<vm::Config, String>,
        Result<smbios::Identity, String>,
    ),
    MaqEdit(MaqField),
    MaqMore,
    MaqReset,
    MaqSave,
    MaqSaved(Result<(), String>),
    MaqRenewAsk(bool),
    MaqRenew,
    MaqRenewed(Result<smbios::Identity, String>),
    MaqDeleteAsk(bool),
    MaqDelete,
    MaqDeleted(Result<(), String>),
}

/// One change on the Máquina screen.
#[derive(Debug, Clone)]
pub enum MaqField {
    Cpus(u32),
    MemoryGib(u32),
    Displays(u32),
    Resolution(String),
    Hidpi(bool),
    Borderless(bool),
    Audio(bool),
    Network(bool),
    SshPort(String),
    StopTimeout(String),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let settings = Settings::load();
        // Images staged by a "Nova máquina" left without creating.
        smbios::clear_staged();
        let now = Instant::now();
        let app = Self {
            locale: Locale::new(&settings.language),
            settings,
            now,
            screen: Screen::Painel,
            entered_at: now,
            report: None,
            vms: Vec::new(),
            passthrough: None,
            host_cpus: vm::host_cpus(),
            host_mem_gib: vm::host_memory_gib(),
            nova: Nova {
                step: 0,
                version: Version::Tahoe,
                name: "mac".into(),
                cpus: 8.min(vm::host_cpus().max(1)),
                memory_gib: 16.min(vm::host_memory_gib().max(2)),
                disk_gib: 128,
                gpu: None,
                switched_at: now,
                busy: false,
                identity: None,
                identity_making: false,
                identity_error: None,
            },
            group: Group::Tahoe,
            group_switched_at: now,
            diag_show_cmd: false,
            diag_fixing: false,
            exec: Exec {
                samples_cpu: vec![0.0; 21],
                samples_mem: vec![0.0; 21],
                last: None,
                stopping: false,
            },
            maquina: Maquina::default(),
            notice: None,
        };
        (app, Task::batch([load(), load_passthrough()]))
    }

    fn title(&self) -> String {
        self.locale.t("app-name")
    }

    pub fn t(&self, key: &str) -> String {
        self.locale.t(key)
    }

    pub fn clock(&self) -> Clock {
        Clock {
            now: self.now,
            reduce_motion: self.settings.reduce_motion,
        }
    }

    fn go(&mut self, screen: Screen) {
        if self.screen != screen {
            self.screen = screen;
            self.entered_at = Instant::now();
            self.diag_show_cmd = false;
            if let Screen::Execucao(_) = screen {
                self.exec = Exec {
                    samples_cpu: vec![0.0; 21],
                    samples_mem: vec![0.0; 21],
                    last: None,
                    stopping: false,
                };
            }
        }
    }

    fn notify(&mut self, text: String) {
        self.notice = Some((Instant::now(), text));
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                self.now = now;
                if let Some((at, _)) = self.notice {
                    if now.duration_since(at) > Duration::from_secs(6) {
                        self.notice = None;
                    }
                }
                Task::none()
            }
            Message::Go(screen) => {
                self.go(screen);
                Task::none()
            }
            Message::Refresh => load(),
            Message::Loaded(report, vms) => {
                self.report = Some(report);
                self.vms = vms;
                self.diag_fixing = false;
                if let Screen::Execucao(i) = self.screen {
                    if self.vms.get(i).map(|v| v.state) != Some(vm::State::Running) {
                        self.go(Screen::Painel);
                    }
                }
                if self.screen == Screen::Maquina
                    && !self.maquina.deleting
                    && !self.vms.iter().any(|v| v.name == self.maquina.name)
                {
                    self.go(Screen::Painel);
                }
                Task::none()
            }
            Message::PassthroughLoaded(p) => {
                self.passthrough = Some(p);
                Task::none()
            }
            Message::LoadFailed(e) => {
                self.diag_fixing = false;
                let msg = self
                    .locale
                    .t_args("err-generic", &[("error", Locale::str(&e))]);
                self.notify(msg);
                Task::none()
            }

            Message::NovaPick(v) => {
                if self.nova.version != v {
                    self.nova.version = v;
                    self.nova.switched_at = Instant::now();
                    let s = v.suggested();
                    self.nova.cpus = s.cpus.min(self.host_cpus.max(1));
                    self.nova.memory_gib = s.memory_gib.min(self.host_mem_gib.max(2));
                    // A GPU that suits one macOS may not suit the next.
                    let keep = self
                        .passthrough
                        .as_ref()
                        .and_then(|p| self.nova.chosen_gpu(&p.1))
                        .is_some_and(|g| g.family.support(v).usable());
                    if !keep {
                        self.nova.gpu = None;
                    }
                }
                Task::none()
            }
            Message::NovaName(n) => {
                self.nova.name = n
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .take(32)
                    .collect();
                Task::none()
            }
            Message::NovaCpus(n) => {
                self.nova.cpus = n.clamp(1, self.host_cpus.max(1));
                Task::none()
            }
            Message::NovaMemory(n) => {
                self.nova.memory_gib = n.clamp(2, self.host_mem_gib.max(2));
                Task::none()
            }
            Message::NovaDisk(n) => {
                self.nova.disk_gib = n.clamp(40, 2048);
                Task::none()
            }
            Message::NovaGpu(g) => {
                self.nova.gpu = g;
                Task::none()
            }
            Message::NovaNext => {
                if self.nova.step < 3 {
                    self.nova.step += 1;
                    self.nova.switched_at = Instant::now();
                }
                if self.nova.step == 3 && self.nova.current_identity().is_none() {
                    return self.stage_identity();
                }
                Task::none()
            }
            Message::NovaIdentityAgain => self.stage_identity(),
            Message::NovaIdentity(result) => {
                self.nova.identity_making = false;
                match result {
                    Ok(s) => {
                        if let Some(old) = self.nova.identity.replace(s) {
                            old.discard();
                        }
                        self.nova.identity_error = None;
                    }
                    Err(e) => self.nova.identity_error = Some(e),
                }
                Task::none()
            }
            Message::NovaBack => {
                if self.nova.step > 0 {
                    self.nova.step -= 1;
                    self.nova.switched_at = Instant::now();
                }
                Task::none()
            }
            Message::NovaCreate => {
                if self.nova.busy || self.nova.name.is_empty() {
                    return Task::none();
                }
                self.nova.busy = true;
                let n = self.nova.clone();
                let pass = self.passthrough.clone();
                let gpu = pass.as_ref().and_then(|p| n.chosen_gpu(&p.1)).cloned();
                let identity = n.current_identity().cloned();
                let advanced = self.settings.advanced.clone();
                // With a GPU, the VM starts only if the host is ready for it already.
                let start = match (&gpu, &pass) {
                    (None, _) => true,
                    (Some(g), Some(p)) => pt::assess(g, n.version, n.memory_gib, &p.0).ready(),
                    (Some(_), None) => false,
                };
                Task::perform(
                    run_blocking(move || {
                        let set = vm::advanced_set_command(&n.name, &advanced);
                        // `create --start` only when nothing has to change before the first boot.
                        let start_at_once = gpu.is_none() && set.is_none();
                        let mut cmd = vm::create_command(
                            &n.name,
                            n.version.slug(),
                            n.cpus,
                            n.memory_gib,
                            n.disk_gib,
                            start_at_once,
                            identity.as_ref(),
                            &advanced,
                        );
                        run_command(&mut cmd)?;
                        // reims-ctl copied the image.
                        if let Some(s) = &identity {
                            s.discard();
                        }
                        if let Some(mut set) = set {
                            run_command(&mut set)?;
                        }
                        if let Some(g) = &gpu {
                            pt::finish_create(&n.name, n.version, g).map_err(|e| e.to_string())?;
                        }
                        if start && !start_at_once {
                            run_command(&mut vm::command(vm::Action::Start, &n.name))?;
                        }
                        Ok(())
                    }),
                    Message::NovaCreated,
                )
            }
            Message::NovaCreated(result) => {
                self.nova.busy = false;
                match result {
                    Ok(()) => {
                        self.nova.step = 0;
                        self.nova.identity = None;
                        self.go(Screen::Painel);
                        load()
                    }
                    Err(e) => {
                        let msg = self
                            .locale
                            .t_args("err-generic", &[("error", Locale::str(&e))]);
                        self.notify(msg);
                        Task::none()
                    }
                }
            }

            Message::DiagFix => {
                let Some(cmd) = self
                    .report
                    .as_ref()
                    .and_then(|r| r.first_problem())
                    .and_then(|c| c.fix_command.clone())
                else {
                    return Task::none();
                };
                self.diag_fixing = true;
                Task::perform(
                    run_blocking(move || run_privileged(&cmd)),
                    Message::DiagFixed,
                )
            }
            Message::DiagFixed(result) => match result {
                Ok(()) => load(),
                Err(e) => {
                    self.diag_fixing = false;
                    let msg = self
                        .locale
                        .t_args("err-generic", &[("error", Locale::str(&e))]);
                    self.notify(msg);
                    Task::none()
                }
            },
            Message::DiagToggleCmd => {
                self.diag_show_cmd = !self.diag_show_cmd;
                Task::none()
            }
            Message::DiagReport => Task::perform(
                run_blocking(|| {
                    cuvee_core::report::write(None)
                        .map(|p| p.display().to_string())
                        .map_err(|e| format!("{e:#}"))
                }),
                Message::DiagReported,
            ),
            Message::DiagReported(result) => {
                let msg = match result {
                    Ok(path) => self
                        .locale
                        .t_args("diag-report-saved", &[("path", Locale::str(&path))]),
                    Err(e) => self
                        .locale
                        .t_args("err-generic", &[("error", Locale::str(&e))]),
                };
                self.notify(msg);
                Task::none()
            }

            Message::PickGroup(g) => {
                if self.group != g {
                    self.group = g;
                    self.group_switched_at = Instant::now();
                }
                Task::none()
            }

            Message::SetLanguage(tag) => {
                if self.settings.language != tag {
                    self.settings.language = tag.to_string();
                    self.locale = Locale::new(tag);
                    self.save_settings();
                }
                Task::none()
            }
            Message::Advanced(edit) => {
                screens::ajustes::apply(&mut self.settings.advanced, edit);
                self.save_settings();
                Task::none()
            }
            Message::SetReduceMotion(on) => {
                self.settings.reduce_motion = on;
                self.save_settings();
                Task::none()
            }

            Message::VmStart(name) => {
                let mut cmd = vm::command(vm::Action::Start, &name);
                Task::perform(
                    run_blocking(move || spawn_detached(&mut cmd)),
                    Message::VmActionDone,
                )
            }
            Message::VmStop(name, force) => {
                self.exec.stopping = true;
                let mut cmd = vm::command(
                    if force {
                        vm::Action::PowerOff
                    } else {
                        vm::Action::Stop
                    },
                    &name,
                );
                Task::perform(
                    run_blocking(move || run_command(&mut cmd)),
                    Message::VmActionDone,
                )
            }
            Message::VmActionDone(result) => {
                self.exec.stopping = false;
                if let Err(e) = result {
                    let msg = self
                        .locale
                        .t_args("err-generic", &[("error", Locale::str(&e))]);
                    self.notify(msg);
                }
                // Give reims-ctl a moment to register the state, then reload.
                Task::perform(
                    async { tokio::time::sleep(Duration::from_millis(800)).await },
                    |_| Message::Refresh,
                )
            }
            Message::MaqOpen(name) => {
                let Some(v) = self.vms.iter().find(|v| v.name == name) else {
                    return Task::none();
                };
                self.maquina = Maquina {
                    name: v.name.clone(),
                    dir: v.dir.clone(),
                    ..Maquina::default()
                };
                self.go(Screen::Maquina);
                load_maquina(&self.maquina)
            }
            Message::MaqLoaded(name, config, identity) => {
                if name != self.maquina.name {
                    return Task::none();
                }
                match config {
                    Ok(c) => {
                        self.maquina.saved = Some(c.clone());
                        self.maquina.edit = Some(c);
                        self.maquina.load_error = None;
                    }
                    Err(e) => self.maquina.load_error = Some(e),
                }
                self.maquina.identity = Some(identity);
                Task::none()
            }
            Message::MaqEdit(field) => {
                let (host_cpus, host_mem) = (self.host_cpus.max(1), self.host_mem_gib.max(2));
                if let Some(c) = self.maquina.edit.as_mut() {
                    let digits =
                        |s: &str| s.chars().filter(char::is_ascii_digit).collect::<String>();
                    match field {
                        MaqField::Cpus(n) => c.cpus = n.clamp(1, host_cpus),
                        MaqField::MemoryGib(n) => c.memory_mib = n.clamp(2, host_mem) as u64 * 1024,
                        MaqField::Displays(n) => c.displays = n.clamp(1, 8),
                        MaqField::Resolution(r) => {
                            c.resolution = r
                                .chars()
                                .filter(|ch| ch.is_ascii_alphanumeric())
                                .take(12)
                                .collect::<String>()
                                .to_ascii_lowercase()
                        }
                        MaqField::Hidpi(b) => c.hidpi = b,
                        MaqField::Borderless(b) => c.borderless = b,
                        MaqField::Audio(b) => c.audio = b,
                        MaqField::Network(b) => c.network = b,
                        MaqField::SshPort(p) => {
                            c.ssh_port = digits(&p).parse::<u32>().unwrap_or(0).min(65535) as u16
                        }
                        MaqField::StopTimeout(t) => {
                            c.stop_timeout = digits(&t).parse::<u64>().unwrap_or(0).min(3600)
                        }
                    }
                }
                Task::none()
            }
            Message::MaqMore => {
                self.maquina.more = !self.maquina.more;
                Task::none()
            }
            Message::MaqReset => {
                self.maquina.edit = self.maquina.saved.clone();
                Task::none()
            }
            Message::MaqSave => {
                let m = &self.maquina;
                let (Some(old), Some(new)) = (&m.saved, &m.edit) else {
                    return Task::none();
                };
                if m.saving || !vm::valid_resolution(&new.resolution) {
                    return Task::none();
                }
                let Some(mut cmd) = vm::set_command(&m.name, old, new) else {
                    return Task::none();
                };
                self.maquina.saving = true;
                Task::perform(
                    run_blocking(move || run_command(&mut cmd)),
                    Message::MaqSaved,
                )
            }
            Message::MaqSaved(result) => {
                self.maquina.saving = false;
                match result {
                    Ok(()) => {
                        self.maquina.saved = self.maquina.edit.clone();
                        let msg = self
                            .locale
                            .t_args("maq-saved", &[("vm", Locale::str(&self.maquina.name))]);
                        self.notify(msg);
                        load_maquina(&self.maquina)
                    }
                    Err(e) => {
                        let msg = self
                            .locale
                            .t_args("err-generic", &[("error", Locale::str(&e))]);
                        self.notify(msg);
                        Task::none()
                    }
                }
            }
            Message::MaqRenewAsk(on) => {
                self.maquina.confirm_renew = on;
                Task::none()
            }
            Message::MaqRenew => {
                if self.maquina.renewing || self.maquina_running() {
                    return Task::none();
                }
                self.maquina.renewing = true;
                self.maquina.confirm_renew = false;
                let (name, dir) = (self.maquina.name.clone(), self.maquina.dir.clone());
                Task::perform(
                    run_blocking(move || {
                        smbios::renew_vm(&name, &dir, None)
                            .map(|r| r.new)
                            .map_err(|e| format!("{e:#}"))
                    }),
                    Message::MaqRenewed,
                )
            }
            Message::MaqRenewed(result) => {
                self.maquina.renewing = false;
                match result {
                    Ok(id) => {
                        self.maquina.identity = Some(Ok(id));
                        let msg = self.locale.t_args(
                            "smbios-new-done",
                            &[("vm", Locale::str(&self.maquina.name))],
                        );
                        self.notify(msg);
                        // reims-ctl set --mac changed vm.toml too.
                        load_maquina(&self.maquina)
                    }
                    Err(e) => {
                        let msg = self
                            .locale
                            .t_args("err-generic", &[("error", Locale::str(&e))]);
                        self.notify(msg);
                        Task::none()
                    }
                }
            }
            Message::MaqDeleteAsk(on) => {
                self.maquina.confirm_delete = on;
                Task::none()
            }
            Message::MaqDelete => {
                if self.maquina.deleting || self.maquina_running() {
                    return Task::none();
                }
                self.maquina.deleting = true;
                let mut cmd = vm::delete_command(&self.maquina.name);
                Task::perform(
                    run_blocking(move || run_command(&mut cmd)),
                    Message::MaqDeleted,
                )
            }
            Message::MaqDeleted(result) => {
                self.maquina.deleting = false;
                self.maquina.confirm_delete = false;
                match result {
                    Ok(()) => {
                        let msg = self
                            .locale
                            .t_args("maq-deleted", &[("vm", Locale::str(&self.maquina.name))]);
                        self.notify(msg);
                        self.go(Screen::Painel);
                        load()
                    }
                    Err(e) => {
                        let msg = self
                            .locale
                            .t_args("err-generic", &[("error", Locale::str(&e))]);
                        self.notify(msg);
                        Task::none()
                    }
                }
            }
            Message::ExecSample => {
                if let Screen::Execucao(i) = self.screen {
                    if let Some(pid) = self.vms.get(i).and_then(|v| v.pid) {
                        let now = Instant::now();
                        match vm::usage(pid) {
                            Some(u) => {
                                if let Some((at, before)) = self.exec.last {
                                    let secs = now.duration_since(at).as_secs_f32();
                                    let cpu = vm::cpu_percent(before, u, secs, self.host_cpus);
                                    push(&mut self.exec.samples_cpu, cpu);
                                    let mem_total =
                                        self.host_mem_gib as f32 * 1024.0 * 1024.0 * 1024.0;
                                    push(
                                        &mut self.exec.samples_mem,
                                        if mem_total > 0.0 {
                                            u.rss_bytes as f32 / mem_total * 100.0
                                        } else {
                                            0.0
                                        },
                                    );
                                }
                                self.exec.last = Some((now, u));
                            }
                            None => return load(),
                        }
                    }
                }
                Task::none()
            }
        }
    }

    /// Whether the VM on the Máquina screen runs now.
    pub fn maquina_running(&self) -> bool {
        self.vms
            .iter()
            .find(|v| v.name == self.maquina.name)
            .is_some_and(|v| v.state == vm::State::Running)
    }

    /// Has reims-util make the new VM's OpenCore image, with a new identity, in the background.
    fn stage_identity(&mut self) -> Task<Message> {
        if self.nova.identity_making {
            return Task::none();
        }
        self.nova.identity_making = true;
        let model = self.nova.wanted_model();
        Task::perform(
            run_blocking(move || smbios::stage(model).map_err(|e| format!("{e:#}"))),
            Message::NovaIdentity,
        )
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.settings.save() {
            let msg = self
                .locale
                .t_args("err-generic", &[("error", Locale::str(&e.to_string()))]);
            self.notify(msg);
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![iced::time::every(Duration::from_secs(5)).map(|_| Message::Refresh)];
        // While something moves (entrances, the breathing dot, the bubbles) tick once per
        // frame the compositor presents. A fixed 16 ms timer gave 60 updates a second that
        // drift against a 144 or 165 Hz display, so motion judders as if it were drawn
        // in software. `window::frames` follows the display's own refresh instead.
        let animating = !self.settings.reduce_motion
            && (matches!(
                self.screen,
                Screen::Painel | Screen::Execucao(_) | Screen::Diagnostico
            ) || !self.clock().is_settled(
                self.entered_at
                    .min(self.nova.switched_at)
                    .min(self.group_switched_at),
                12,
            ) || self.notice.is_some());
        if animating {
            subs.push(iced::window::frames().map(Message::Tick));
        } else {
            subs.push(iced::time::every(Duration::from_millis(250)).map(Message::Tick));
        }
        if let Screen::Execucao(_) = self.screen {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::ExecSample));
        }
        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        let t = |k: &str| self.t(k);
        let body: Element<'_, Message> = match self.screen {
            Screen::Painel => screens::painel::view(self),
            Screen::Nova => screens::nova::view(self),
            Screen::Diagnostico => screens::diagnostico::view(self),
            Screen::Comportamento => screens::comportamento::view(self),
            Screen::Ajustes => screens::ajustes::view(self),
            Screen::Execucao(i) => screens::execucao::view(self, i),
            Screen::Maquina => screens::maquina::view(self),
        };
        let main = container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([48, 72]);
        let page = row![widgets::rail::view(self.screen, &t), main].height(Length::Fill);
        match &self.notice {
            Some((_, text)) => iced::widget::stack![
                page,
                container(
                    container(theme::label(text.clone(), 13.0, theme::INK))
                        .padding([12, 18])
                        .style(theme::code_block)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .padding(24),
            ]
            .into(),
            None => page.into(),
        }
    }
}

fn app_theme(_: &App) -> iced::Theme {
    theme::theme()
}

fn app_style(_: &App, t: &iced::Theme) -> iced::theme::Style {
    theme::app_style(t)
}

fn push(samples: &mut Vec<f32>, v: f32) {
    samples.push(v);
    if samples.len() > 21 {
        samples.remove(0);
    }
}

fn load_passthrough() -> Task<Message> {
    Task::perform(
        run_blocking(|| {
            let sys = std::path::Path::new(pt::SYSFS);
            let devices = pt::pci::scan(sys);
            Arc::new((pt::host_state(sys, &devices), pt::gpus(&devices)))
        }),
        Message::PassthroughLoaded,
    )
}

/// Reads the VM's vm.toml and the Mac its OpenCore image presents, in the background.
fn load_maquina(m: &Maquina) -> Task<Message> {
    let (name, dir) = (m.name.clone(), m.dir.clone());
    Task::perform(
        run_blocking(move || {
            let config = vm::read_config(&dir).map_err(|e| format!("{e:#}"));
            let identity = smbios::read_vm(&dir).map_err(|e| format!("{e:#}"));
            (name, config, identity)
        }),
        |(name, config, identity)| Message::MaqLoaded(name, config, identity),
    )
}

fn load() -> Task<Message> {
    Task::perform(
        run_blocking(|| {
            let report = host::inspect();
            let vms = vm::list();
            (Arc::new(report), vms)
        }),
        |(report, vms)| match vms {
            Ok(vms) => Message::Loaded(report, vms),
            Err(e) => Message::LoadFailed(e.to_string()),
        },
    )
}

async fn run_blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    match tokio::task::spawn_blocking(f).await {
        Ok(v) => v,
        Err(e) => std::panic::resume_unwind(e.into_panic()),
    }
}

fn run_command(cmd: &mut std::process::Command) -> Result<(), String> {
    let out = cmd
        .output()
        .map_err(|e| format!("{}: {e}", vm::display(cmd)))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(err.lines().last().unwrap_or("").trim().to_string())
    }
}

fn spawn_detached(cmd: &mut std::process::Command) -> Result<(), String> {
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{}: {e}", vm::display(cmd)))
}

/// Runs a fix that needs root through polkit (`pkexec`), which shows the desktop's own
/// password prompt. `cuvee setup-host` refuses root, so it is run as the user with sudo inside.
fn run_privileged(command: &str) -> Result<(), String> {
    let mut c = if command.starts_with("cuvee ") {
        let mut c = std::process::Command::new("pkexec");
        c.args(["--user", &whoami()]).args(["sh", "-c", command]);
        c
    } else {
        let mut c = std::process::Command::new("pkexec");
        c.args(["sh", "-c", command.trim_start_matches("sudo ")]);
        c
    };
    run_command(&mut c)
}

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "root".into())
}
