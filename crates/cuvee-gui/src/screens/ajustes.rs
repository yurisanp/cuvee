//! Ajustes: language (a sliding pill), animations, the advanced choices for new machines
//! (closed until asked for), and about.

use crate::theme;
use crate::widgets;
use crate::{App, Message};
use cuvee_core::i18n::{Locale, LANGUAGES};
use cuvee_core::settings::{valid_resolution, Advanced};
use iced::widget::{button, column, container, row, scrollable, text_input, Column, Row};
use iced::{Element, Length};
use std::path::PathBuf;

/// A change to the advanced choices.
#[derive(Debug, Clone)]
pub enum Edit {
    Open(bool),
    Displays(Option<u32>),
    Resolution(Option<String>),
    Hidpi(Option<bool>),
    Borderless(Option<bool>),
    Refresh(Option<u32>),
    Gpu(String),
    VideoAcceleration(Option<bool>),
    Audio(Option<bool>),
    AudioBackend(Option<&'static str>),
    Network(Option<bool>),
    SshPort(String),
    Logs(Option<bool>),
    LogLevel(String),
    StopTimeout(String),
    Offline(bool),
    Folder(String),
    ExtraArgs(String),
    Env(String),
    Reset,
}

fn some_text(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

pub fn apply(a: &mut Advanced, edit: Edit) {
    match edit {
        Edit::Open(on) => a.open = on,
        Edit::Displays(n) => a.displays = n,
        Edit::Resolution(r) => a.resolution = r,
        Edit::Hidpi(v) => a.hidpi = v,
        Edit::Borderless(v) => a.borderless = v,
        Edit::Refresh(n) => a.refresh = n,
        Edit::Gpu(g) => a.gpu = some_text(g),
        Edit::VideoAcceleration(v) => a.video_acceleration = v,
        Edit::Audio(v) => a.audio = v,
        Edit::AudioBackend(b) => a.audio_backend = b.map(String::from),
        Edit::Network(v) => a.network = v,
        Edit::SshPort(s) => {
            let digits: String = s.chars().filter(char::is_ascii_digit).take(5).collect();
            if digits.is_empty() {
                a.ssh_port = None;
            } else if let Ok(n) = digits.parse() {
                a.ssh_port = Some(n);
            }
        }
        Edit::Logs(v) => a.logs = v,
        Edit::LogLevel(l) => a.log_level = some_text(l),
        Edit::StopTimeout(s) => {
            let digits: String = s.chars().filter(char::is_ascii_digit).take(5).collect();
            a.stop_timeout = digits.parse().ok();
        }
        Edit::Offline(on) => a.offline = on,
        Edit::Folder(f) => a.folder = some_text(f).map(PathBuf::from),
        Edit::ExtraArgs(e) => a.extra_args = e,
        Edit::Env(e) => a.env = e,
        Edit::Reset => a.reset(),
    }
}

/// The presets offered for the first mode; anything else is "other".
const RESOLUTIONS: [&str; 3] = ["host", "1920x1080", "2560x1440"];
const REFRESH: [u32; 5] = [0, 60, 120, 144, 165];
const AUDIO_BACKENDS: [&str; 5] = ["auto", "pipewire", "pa", "alsa", "none"];

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let r0 = clock.rise(t0, 0);
    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);
    let r3 = clock.rise(t0, 3);
    let r4 = clock.rise(t0, 4);

    // Language pill: the chosen language sits on a champagne capsule.
    let mut pill = Row::new().spacing(4).align_y(iced::Center);
    for lang in LANGUAGES {
        let on = lang.tag == app.locale.tag();
        let label = theme::label(
            lang.native_name,
            14.0,
            if on { theme::BG } else { theme::MUTED },
        )
        .font(theme::SANS_MEDIUM);
        let b = if on {
            container(label)
                .padding([10, 18])
                .style(theme::tinted(theme::ACCENT))
        } else {
            container(
                button(label)
                    .padding([10, 18])
                    .style(theme::ghost)
                    .on_press(Message::SetLanguage(lang.tag)),
            )
        };
        pill = pill.push(b);
    }
    let pill = container(pill)
        .padding(4)
        .style(theme::tinted(theme::HOVER_BG));

    let motion = row![
        toggle(
            app,
            app.t("ajustes-motion-on"),
            !app.settings.reduce_motion,
            Message::SetReduceMotion(false)
        ),
        toggle(
            app,
            app.t("ajustes-motion-off"),
            app.settings.reduce_motion,
            Message::SetReduceMotion(true)
        ),
    ]
    .spacing(4);
    let motion = container(motion)
        .padding(4)
        .style(theme::tinted(theme::HOVER_BG));

    let page = column![
        widgets::risen(
            r0,
            theme::serif(
                app.t("ajustes-title"),
                56.0,
                theme::with_alpha(theme::INK, r0.opacity)
            )
        ),
        widgets::hairline(),
        widgets::risen(
            r1,
            section(app.t("ajustes-language"), pill.into(), r1.opacity)
        ),
        widgets::risen(
            r2,
            section(
                app.t("ajustes-motion"),
                column![
                    motion,
                    theme::label(
                        app.t("ajustes-motion-hint"),
                        13.0,
                        theme::with_alpha(theme::SOFT, r2.opacity)
                    )
                ]
                .spacing(10)
                .into(),
                r2.opacity
            )
        ),
        widgets::risen(r3, advanced(app, r3.opacity)),
        widgets::risen(
            r4,
            section(
                app.t("ajustes-about"),
                theme::label(
                    app.locale.t_args(
                        "ajustes-about-text",
                        &[("version", Locale::str(env!("CARGO_PKG_VERSION")))]
                    ),
                    14.0,
                    theme::with_alpha(theme::SOFT, r4.opacity)
                )
                .into(),
                r4.opacity
            )
        ),
    ]
    .spacing(32)
    .max_width(760)
    .width(Length::Fill);
    scrollable(container(page).padding(iced::Padding {
        right: 24.0,
        ..iced::Padding::ZERO
    }))
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(3).scroller_width(3),
    ))
    .height(Length::Fill)
    .into()
}

/// One pill in a segmented choice: the chosen one on champagne, the others as ghost buttons.
fn pill<'a>(label: String, on: bool, msg: Message) -> Element<'a, Message> {
    let text = theme::label(label, 13.0, if on { theme::BG } else { theme::MUTED })
        .font(theme::SANS_MEDIUM);
    if on {
        container(text)
            .padding([7, 14])
            .style(theme::tinted(theme::ACCENT))
            .into()
    } else {
        button(text)
            .padding([7, 14])
            .style(theme::ghost)
            .on_press(msg)
            .into()
    }
}

fn segmented<'a>(pills: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    container(Row::with_children(pills).spacing(2).wrap())
        .padding(3)
        .style(theme::tinted(theme::HOVER_BG))
        .into()
}

/// Default / on / off, for a choice reims-ctl makes when left alone.
fn tri<'a>(app: &App, value: Option<bool>, edit: fn(Option<bool>) -> Edit) -> Element<'a, Message> {
    segmented(vec![
        pill(
            app.t("adv-default"),
            value.is_none(),
            Message::Advanced(edit(None)),
        ),
        pill(
            app.t("adv-on"),
            value == Some(true),
            Message::Advanced(edit(Some(true))),
        ),
        pill(
            app.t("adv-off"),
            value == Some(false),
            Message::Advanced(edit(Some(false))),
        ),
    ])
}

fn input<'a>(
    placeholder: &str,
    value: &str,
    edit: fn(String) -> Edit,
    mono: bool,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(move |s| Message::Advanced(edit(s)))
        .size(14)
        .font(if mono { theme::MONO } else { theme::SANS })
        .padding([8, 10])
        .into()
}

/// A label (with an optional hint under it) and its control, side by side.
fn field<'a>(
    label: String,
    hint: Option<String>,
    control: Element<'a, Message>,
    a: f32,
) -> Element<'a, Message> {
    let mut left = column![theme::label(label, 14.0, theme::with_alpha(theme::INK, a))].spacing(3);
    if let Some(h) = hint {
        left = left.push(theme::label(h, 12.0, theme::with_alpha(theme::SOFT, a)));
    }
    row![
        container(left).width(Length::Fixed(200.0)),
        container(control).width(Length::Fill)
    ]
    .spacing(20)
    .align_y(iced::Alignment::Start)
    .into()
}

fn group<'a>(title: String, fields: Vec<Element<'a, Message>>, a: f32) -> Element<'a, Message> {
    let mut col = Column::new().spacing(18).push(theme::label(
        title,
        12.0,
        theme::with_alpha(theme::ACCENT, a),
    ));
    for f in fields {
        col = col.push(f);
    }
    col.into()
}

/// The advanced choices: one line and a switch while closed, every reims-ctl knob once open.
fn advanced(app: &App, a: f32) -> Element<'_, Message> {
    let adv = &app.settings.advanced;
    let head = row![
        theme::label(app.t("adv-title"), 12.0, theme::with_alpha(theme::MUTED, a))
            .width(Length::Fill),
        button(
            theme::label(
                app.t(if adv.open { "adv-hide" } else { "adv-show" }),
                13.0,
                theme::with_alpha(theme::ACCENT, a)
            )
            .font(theme::SANS_MEDIUM)
        )
        .padding([4, 0])
        .style(theme::ghost_bright)
        .on_press(Message::Advanced(Edit::Open(!adv.open))),
    ]
    .align_y(iced::Center);
    let summary = if adv.is_default() {
        app.t("adv-summary-none")
    } else {
        app.t("adv-summary-some")
    };
    let intro = column![
        head,
        theme::label(summary, 13.0, theme::with_alpha(theme::SOFT, a))
    ]
    .spacing(8);
    if !adv.open {
        return intro.into();
    }
    let mut col = column![intro];

    // Tela
    let mut displays = vec![pill(
        app.t("adv-default"),
        adv.displays.is_none(),
        Message::Advanced(Edit::Displays(None)),
    )];
    for n in 1..=4 {
        displays.push(pill(
            n.to_string(),
            adv.displays == Some(n),
            Message::Advanced(Edit::Displays(Some(n))),
        ));
    }
    let custom_res = adv
        .resolution
        .as_deref()
        .is_some_and(|r| !RESOLUTIONS.contains(&r));
    let mut res = vec![pill(
        app.t("adv-default"),
        adv.resolution.is_none(),
        Message::Advanced(Edit::Resolution(None)),
    )];
    for r in RESOLUTIONS {
        let label = if r == "host" {
            app.t("adv-resolution-host")
        } else {
            r.replace('x', "×")
        };
        res.push(pill(
            label,
            adv.resolution.as_deref() == Some(r),
            Message::Advanced(Edit::Resolution(Some(r.to_string()))),
        ));
    }
    res.push(pill(
        app.t("adv-other"),
        custom_res,
        Message::Advanced(Edit::Resolution(Some(String::new()))),
    ));
    let mut res_col = column![segmented(res)].spacing(8);
    if custom_res {
        let value = adv.resolution.clone().unwrap_or_default();
        res_col = res_col.push(
            text_input("2048x1152", &value)
                .on_input(|s| Message::Advanced(Edit::Resolution(Some(s))))
                .size(14)
                .font(theme::MONO)
                .padding([8, 10]),
        );
        if !value.is_empty() && !valid_resolution(&value) {
            res_col = res_col.push(theme::label(
                app.t("adv-resolution-bad"),
                12.0,
                theme::with_alpha(theme::WARN, a),
            ));
        }
    }
    let mut refresh = vec![pill(
        app.t("adv-default"),
        adv.refresh.is_none(),
        Message::Advanced(Edit::Refresh(None)),
    )];
    for hz in REFRESH {
        let label = if hz == 0 {
            app.t("adv-refresh-host")
        } else {
            format!("{hz} Hz")
        };
        refresh.push(pill(
            label,
            adv.refresh == Some(hz),
            Message::Advanced(Edit::Refresh(Some(hz))),
        ));
    }
    let screen = group(
        app.t("adv-group-display"),
        vec![
            field(
                app.t("adv-displays"),
                Some(app.t("adv-displays-hint")),
                segmented(displays),
                a,
            ),
            field(app.t("adv-resolution"), None, res_col.into(), a),
            field(
                app.t("adv-hidpi"),
                Some(app.t("adv-hidpi-hint")),
                tri(app, adv.hidpi, Edit::Hidpi),
                a,
            ),
            field(
                app.t("adv-borderless"),
                None,
                tri(app, adv.borderless, Edit::Borderless),
                a,
            ),
            field(app.t("adv-refresh"), None, segmented(refresh), a),
        ],
        a,
    );

    // GPU e vídeo
    let video = group(
        app.t("adv-group-gpu"),
        vec![
            field(
                app.t("adv-gpu"),
                Some(app.t("adv-gpu-hint")),
                input("auto", adv.gpu.as_deref().unwrap_or(""), Edit::Gpu, true),
                a,
            ),
            field(
                app.t("adv-video-accel"),
                Some(app.t("adv-video-accel-hint")),
                tri(app, adv.video_acceleration, Edit::VideoAcceleration),
                a,
            ),
        ],
        a,
    );

    // Som e rede
    let mut backends = vec![pill(
        app.t("adv-default"),
        adv.audio_backend.is_none(),
        Message::Advanced(Edit::AudioBackend(None)),
    )];
    for b in AUDIO_BACKENDS {
        backends.push(pill(
            b.to_string(),
            adv.audio_backend.as_deref() == Some(b),
            Message::Advanced(Edit::AudioBackend(Some(b))),
        ));
    }
    let ssh = adv.ssh_port.map(|p| p.to_string()).unwrap_or_default();
    let sound = group(
        app.t("adv-group-io"),
        vec![
            field(
                app.t("adv-audio"),
                None,
                tri(app, adv.audio, Edit::Audio),
                a,
            ),
            field(app.t("adv-audio-backend"), None, segmented(backends), a),
            field(
                app.t("adv-network"),
                None,
                tri(app, adv.network, Edit::Network),
                a,
            ),
            field(
                app.t("adv-ssh"),
                Some(app.t("adv-ssh-hint")),
                input("2222", &ssh, Edit::SshPort, true),
                a,
            ),
        ],
        a,
    );

    // Sistema
    let folder = adv
        .folder
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let timeout = adv.stop_timeout.map(|n| n.to_string()).unwrap_or_default();
    let system = group(
        app.t("adv-group-system"),
        vec![
            field(
                app.t("adv-offline"),
                Some(app.t("adv-offline-hint")),
                segmented(vec![
                    pill(
                        app.t("adv-offline-no"),
                        !adv.offline,
                        Message::Advanced(Edit::Offline(false)),
                    ),
                    pill(
                        app.t("adv-offline-yes"),
                        adv.offline,
                        Message::Advanced(Edit::Offline(true)),
                    ),
                ]),
                a,
            ),
            field(
                app.t("adv-folder"),
                Some(app.t("adv-folder-hint")),
                input("~/.local/share/reims-ctl/vms", &folder, Edit::Folder, true),
                a,
            ),
            field(
                app.t("adv-stop-timeout"),
                Some(app.t("adv-stop-timeout-hint")),
                input("60", &timeout, Edit::StopTimeout, true),
                a,
            ),
            field(app.t("adv-logs"), None, tri(app, adv.logs, Edit::Logs), a),
            field(
                app.t("adv-log-level"),
                None,
                input(
                    "info",
                    adv.log_level.as_deref().unwrap_or(""),
                    Edit::LogLevel,
                    true,
                ),
                a,
            ),
        ],
        a,
    );

    // QEMU
    let qemu = group(
        app.t("adv-group-qemu"),
        vec![
            field(
                app.t("adv-extra-args"),
                Some(app.t("adv-extra-args-hint")),
                input("-device usb-tablet", &adv.extra_args, Edit::ExtraArgs, true),
                a,
            ),
            field(
                app.t("adv-env"),
                Some(app.t("adv-env-hint")),
                input("MESA_DEBUG=1", &adv.env, Edit::Env, true),
                a,
            ),
        ],
        a,
    );

    col = col
        .push(screen)
        .push(video)
        .push(sound)
        .push(system)
        .push(qemu);

    // What the choices turn into, as reims-ctl flags.
    let mut flags = adv.create_args("NAME");
    flags.extend(adv.set_args());
    if !flags.is_empty() {
        let line = flags
            .iter()
            .map(|f| cuvee_core::passthrough::sh_quote(f))
            .collect::<Vec<_>>()
            .join(" ");
        col = col.push(
            column![
                theme::label(
                    app.t("adv-preview"),
                    12.0,
                    theme::with_alpha(theme::MUTED, a)
                ),
                container(theme::mono(line, 12.0, theme::with_alpha(theme::SOFT, a)))
                    .padding([10, 14])
                    .width(Length::Fill)
                    .style(theme::code_block),
                button(theme::label(
                    app.t("adv-reset"),
                    13.0,
                    theme::with_alpha(theme::MUTED, a)
                ))
                .padding([4, 0])
                .style(theme::ghost)
                .on_press(Message::Advanced(Edit::Reset)),
            ]
            .spacing(8),
        );
    }
    col.spacing(28).into()
}

fn section<'a>(title: String, body: Element<'a, Message>, a: f32) -> Element<'a, Message> {
    column![
        theme::label(title, 12.0, theme::with_alpha(theme::MUTED, a)),
        body
    ]
    .spacing(12)
    .into()
}

fn toggle<'a>(_app: &App, label: String, on: bool, msg: Message) -> Element<'a, Message> {
    let text = theme::label(label, 14.0, if on { theme::BG } else { theme::MUTED })
        .font(theme::SANS_MEDIUM);
    if on {
        container(text)
            .padding([10, 18])
            .style(theme::tinted(theme::ACCENT))
            .into()
    } else {
        button(text)
            .padding([10, 18])
            .style(theme::ghost)
            .on_press(msg)
            .into()
    }
}
