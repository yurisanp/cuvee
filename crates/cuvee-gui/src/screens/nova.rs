//! Nova máquina: four steps. System (which macOS), Hardware, Disk, Review.

use crate::theme;
use crate::widgets::{self, captioned};
use crate::{App, Message};
use cuvee_core::i18n::Locale;
use cuvee_core::macos::{Installer, Version};
use cuvee_core::vm;
use iced::widget::{button, column, container, row, rule, slider, text_input, Row};
use iced::{Element, Length};
use std::time::Duration;

const STEPS: [&str; 4] = [
    "nova-step-system",
    "nova-step-hardware",
    "nova-step-disk",
    "nova-step-review",
];

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let n = &app.nova;

    // Progress: a long bar for the current step, short ones for the rest.
    let mut bars = Row::new().spacing(10).align_y(iced::Center);
    for (i, _) in STEPS.iter().enumerate() {
        let (w, c) = if i == n.step {
            (28.0, theme::ACCENT)
        } else {
            (12.0, theme::LINE_STRONG)
        };
        bars = bars.push(
            container(rule::horizontal(3).style(theme::hairline_colored(c)))
                .width(Length::Fixed(w)),
        );
    }
    bars =
        bars.push(widgets::hgap(8.0))
            .push(theme::label(app.t(STEPS[n.step]), 13.0, theme::MUTED));
    let r0 = clock.rise(t0, 0);

    let body: Element<'_, Message> = match n.step {
        0 => step_system(app),
        1 => step_hardware(app),
        2 => step_disk(app),
        _ => step_review(app),
    };

    let footer = row![
        button(theme::label(
            app.t(if n.step == 0 {
                "nova-cancel"
            } else {
                "nova-back"
            }),
            14.0,
            theme::MUTED
        ))
        .padding([0, 2])
        .style(theme::ghost)
        .on_press(if n.step == 0 {
            Message::Go(crate::Screen::Painel)
        } else {
            Message::NovaBack
        }),
        iced::widget::space().width(Length::Fill),
        if n.step < 3 {
            button(theme::label(app.t("nova-continue"), 15.0, theme::BG).font(theme::SANS_MEDIUM))
                .padding([14, 26])
                .style(theme::cta)
                .on_press(Message::NovaNext)
        } else {
            button(
                theme::label(
                    if n.busy {
                        "…".into()
                    } else {
                        app.t("nova-create")
                    },
                    15.0,
                    theme::BG,
                )
                .font(theme::SANS_MEDIUM),
            )
            .padding([14, 26])
            .style(theme::cta)
            .on_press_maybe((!n.busy && !n.name.is_empty()).then_some(Message::NovaCreate))
        },
    ]
    .align_y(iced::Center);

    column![
        widgets::risen(r0, bars),
        body,
        iced::widget::space().height(Length::Fill),
        footer
    ]
    .spacing(36)
    .height(Length::Fill)
    .into()
}

fn step_system(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let n = &app.nova;
    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);

    let mut picker = Row::new()
        .spacing(30)
        .align_y(iced::alignment::Vertical::Bottom);
    for v in Version::ALL {
        let on = v == n.version;
        // The underline grows under the chosen version over 450 ms.
        let grow = if on {
            clock.progress(n.switched_at, Duration::ZERO, Duration::from_millis(450))
        } else {
            0.0
        };
        let color = if on { theme::INK } else { theme::DIM };
        let item = column![
            theme::serif(v.number(), 40.0, theme::with_alpha(color, r2.opacity)),
            container(rule::horizontal(2).style(theme::hairline_colored(theme::ACCENT)))
                .width(Length::FillPortion(1))
                .padding(iced::Padding {
                    top: 6.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0
                })
                .width(Length::Fixed(grow * 100.0))
                .height(Length::Fixed(8.0)),
        ]
        .spacing(0);
        picker = picker.push(
            button(item)
                .padding([8, 2])
                .style(theme::bare)
                .on_press(Message::NovaPick(v)),
        );
    }

    // The selection panel swaps with a fade-and-rise.
    let swap = clock.rise(n.switched_at, 0);
    let a = swap.opacity;
    let v = n.version;
    let (tag, tag_color) = if v.accelerated() {
        (app.t("nova-tag-accelerated"), theme::ACCENT)
    } else {
        (app.t("nova-tag-plain"), theme::WARN)
    };
    let video = app.t(if v.accelerated() {
        "nova-video-metal"
    } else {
        "nova-video-plain"
    });
    let installer = app.t(match v.installer() {
        Installer::OnlineOrOffline => "nova-installer-both",
        Installer::OnlineOnly => "nova-installer-online",
    });
    let s = v.suggested();
    let hw = app.locale.t_args(
        "nova-hw",
        &[
            ("cpus", Locale::num(s.cpus as f64)),
            ("gib", Locale::num(s.memory_gib as f64)),
        ],
    );

    let panel = column![
        row![
            theme::serif(v.name(), 34.0, theme::with_alpha(theme::INK, a))
                .font(theme::SERIF_REGULAR),
            theme::label(tag, 14.0, theme::with_alpha(tag_color, a)),
        ]
        .spacing(16)
        .align_y(iced::alignment::Vertical::Bottom),
        widgets::hairline(),
        row![
            container(captioned(app.t("nova-video"), video, a)).width(Length::Fill),
            container(captioned(app.t("nova-installer"), installer, a)).width(Length::Fill),
            container(captioned(app.t("nova-suggested"), hw, a)).width(Length::Fill),
        ]
        .spacing(24),
        theme::label(app.t(v.tip_key()), 15.0, theme::with_alpha(theme::SOFT, a)).line_height(1.6),
    ]
    .spacing(22)
    .max_width(760);

    column![
        widgets::risen(
            r1,
            theme::serif(
                app.t("nova-title"),
                56.0,
                theme::with_alpha(theme::INK, r1.opacity)
            )
        ),
        widgets::risen(r2, picker),
        widgets::risen(swap, panel),
    ]
    .spacing(36)
    .into()
}

fn labeled_slider<'a>(
    label: String,
    value_text: String,
    range: std::ops::RangeInclusive<u32>,
    value: u32,
    on_change: impl Fn(u32) -> Message + 'a,
    alpha: f32,
) -> Element<'a, Message> {
    column![
        row![
            theme::label(label, 13.0, theme::with_alpha(theme::MUTED, alpha)),
            iced::widget::space().width(Length::Fill),
            theme::serif(value_text, 30.0, theme::with_alpha(theme::INK, alpha)),
        ]
        .align_y(iced::alignment::Vertical::Bottom),
        slider(range, value, on_change).step(1u32),
    ]
    .spacing(10)
    .into()
}

fn step_hardware(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let n = &app.nova;
    let r1 = clock.rise(n.switched_at, 0);
    let r2 = clock.rise(n.switched_at, 1);
    let a = r2.opacity;
    let advice_cpu = if n.cpus + 2 > app.host_cpus {
        app.locale.t_args(
            "ajustes-advice-cpus-high",
            &[("host", Locale::num(app.host_cpus as f64))],
        )
    } else {
        app.t("ajustes-advice-cpus-ok")
    };
    let advice_mem = if n.memory_gib * 2 > app.host_mem_gib {
        app.locale.t_args(
            "ajustes-advice-mem-high",
            &[("host", Locale::num(app.host_mem_gib as f64))],
        )
    } else if n.memory_gib < 8 && n.version.accelerated() {
        app.t("ajustes-advice-mem-low")
    } else {
        app.t("ajustes-advice-mem-ok")
    };
    column![
        widgets::risen(
            r1,
            theme::serif(
                app.t("nova-step-hardware"),
                56.0,
                theme::with_alpha(theme::INK, r1.opacity)
            )
        ),
        widgets::risen(
            r2,
            column![
                column![
                    theme::label(app.t("nova-name"), 13.0, theme::with_alpha(theme::MUTED, a)),
                    text_input("mac", &n.name)
                        .on_input(Message::NovaName)
                        .size(22)
                        .font(theme::SERIF_REGULAR)
                        .padding(10),
                ]
                .spacing(10),
                labeled_slider(
                    app.t("nova-cpus"),
                    n.cpus.to_string(),
                    1..=app.host_cpus.max(1),
                    n.cpus,
                    Message::NovaCpus,
                    a
                ),
                theme::label(advice_cpu, 13.0, theme::with_alpha(theme::SOFT, a)),
                labeled_slider(
                    app.t("nova-memory"),
                    format!("{} GB", n.memory_gib),
                    2..=app.host_mem_gib.max(2),
                    n.memory_gib,
                    Message::NovaMemory,
                    a
                ),
                theme::label(advice_mem, 13.0, theme::with_alpha(theme::SOFT, a)),
            ]
            .spacing(22)
            .max_width(640)
        ),
    ]
    .spacing(36)
    .into()
}

fn step_disk(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let n = &app.nova;
    let r1 = clock.rise(n.switched_at, 0);
    let r2 = clock.rise(n.switched_at, 1);
    let a = r2.opacity;
    column![
        widgets::risen(
            r1,
            theme::serif(
                app.t("nova-step-disk"),
                56.0,
                theme::with_alpha(theme::INK, r1.opacity)
            )
        ),
        widgets::risen(
            r2,
            column![
                labeled_slider(
                    app.t("nova-disk-size"),
                    format!("{} GB", n.disk_gib),
                    40..=1024,
                    n.disk_gib,
                    Message::NovaDisk,
                    a
                ),
                theme::label(
                    app.t("nova-disk-hint"),
                    13.0,
                    theme::with_alpha(theme::SOFT, a)
                ),
            ]
            .spacing(22)
            .max_width(640)
        ),
    ]
    .spacing(36)
    .into()
}

fn step_review(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let n = &app.nova;
    let r1 = clock.rise(n.switched_at, 0);
    let r2 = clock.rise(n.switched_at, 1);
    let a = r2.opacity;
    let cmd = vm::create_command(
        &n.name,
        n.version.slug(),
        n.cpus,
        n.memory_gib,
        n.disk_gib,
        true,
    );
    let hw = app.locale.t_args(
        "nova-hw",
        &[
            ("cpus", Locale::num(n.cpus as f64)),
            ("gib", Locale::num(n.memory_gib as f64)),
        ],
    );
    column![
        widgets::risen(
            r1,
            theme::serif(
                app.t("nova-review-title"),
                56.0,
                theme::with_alpha(theme::INK, r1.opacity)
            )
        ),
        widgets::risen(
            r2,
            column![
                row![
                    container(captioned(app.t("nova-name"), n.name.clone(), a)).width(Length::Fill),
                    container(captioned(
                        app.t("nova-step-system"),
                        format!("macOS {} {}", n.version.number(), n.version.name()),
                        a
                    ))
                    .width(Length::Fill),
                    container(captioned(app.t("nova-step-hardware"), hw, a)).width(Length::Fill),
                    container(captioned(
                        app.t("nova-disk-size"),
                        format!("{} GB", n.disk_gib),
                        a
                    ))
                    .width(Length::Fill),
                ]
                .spacing(24),
                widgets::hairline(),
                theme::label(
                    app.t("nova-review-command"),
                    13.0,
                    theme::with_alpha(theme::MUTED, a)
                ),
                container(theme::mono(
                    format!("$ {}", vm::display(&cmd)),
                    13.0,
                    theme::SOFT
                ))
                .padding([10, 14])
                .style(theme::code_block),
                theme::label(
                    app.t(n.version.tip_key()),
                    15.0,
                    theme::with_alpha(theme::SOFT, a)
                )
                .line_height(1.6),
            ]
            .spacing(22)
            .max_width(760)
        ),
    ]
    .spacing(36)
    .into()
}
