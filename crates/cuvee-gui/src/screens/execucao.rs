//! Em execução: one running VM, its clock, CPU and memory, and the power controls.

use crate::theme;
use crate::widgets::{self, bubbles, dot, icons, spark};
use crate::{App, Message, Screen};
use iced::widget::{button, column, container, row, stack};
use iced::{Element, Length};

pub fn view(app: &App, index: usize) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let Some(vm) = app.vms.get(index) else {
        return container(theme::label(app.t("exec-back"), 14.0, theme::MUTED)).into();
    };

    let r0 = clock.rise(t0, 0);
    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);

    let uptime = vm
        .pid
        .and_then(cuvee_core::vm::uptime)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let clock_text = format!(
        "{:02}:{:02}:{:02}",
        uptime / 3600,
        uptime / 60 % 60,
        uptime % 60
    );
    let (state_color, state_text) = if app.exec.stopping {
        (theme::WARN, app.t("exec-stopping"))
    } else {
        (theme::GOOD, app.t("vm-state-running"))
    };

    let header = column![
        row![
            dot::dot(
                Some(theme::with_alpha(state_color, r0.opacity)),
                if app.exec.stopping {
                    0.0
                } else {
                    clock.breathe(t0, 2.4)
                }
            ),
            theme::label(state_text, 14.0, theme::with_alpha(state_color, r0.opacity)),
            theme::label(" · ", 14.0, theme::with_alpha(theme::MUTED, r0.opacity)),
            theme::mono(
                clock_text,
                14.0,
                theme::with_alpha(theme::MUTED, r0.opacity)
            ),
        ]
        .spacing(4)
        .align_y(iced::Center),
        theme::serif(
            vm.name.clone(),
            88.0,
            theme::with_alpha(theme::INK, r0.opacity)
        ),
        theme::label(
            format!("{} · {}", vm.dir.display(), app.t("vm-accelerated")),
            15.0,
            theme::with_alpha(theme::MUTED, r0.opacity)
        ),
    ]
    .spacing(14);

    let hero = stack![
        container(bubbles::bubbles(
            clock.seconds(t0),
            if app.exec.stopping { 0.0 } else { 1.0 },
            360.0,
            300.0
        ))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right),
        widgets::risen(r0, header),
    ];

    let a = r1.opacity;
    let last_cpu = app.exec.samples_cpu.last().copied().unwrap_or(0.0);
    let last_mem = app.exec.samples_mem.last().copied().unwrap_or(0.0);
    let meter = |label: String,
                 value: String,
                 samples: Vec<f32>,
                 color: iced::Color|
     -> Element<'_, Message> {
        column![
            row![
                theme::label(label, 13.0, theme::with_alpha(theme::MUTED, a)),
                iced::widget::space().width(Length::Fill),
                theme::serif(value, 30.0, theme::with_alpha(theme::INK, a)),
            ]
            .align_y(iced::alignment::Vertical::Bottom),
            spark::spark(samples, theme::with_alpha(color, a)),
        ]
        .spacing(10)
        .width(Length::Fill)
        .into()
    };
    let meters = row![
        meter(
            app.t("exec-cpu"),
            format!("{last_cpu:.0}%"),
            app.exec.samples_cpu.clone(),
            theme::ACCENT
        ),
        meter(
            app.t("exec-host-gpu"),
            "—".into(),
            vec![0.0; 21],
            theme::INFO
        ),
        meter(
            app.t("exec-memory"),
            format!("{last_mem:.0}%"),
            app.exec.samples_mem.clone(),
            theme::SOFT
        ),
    ]
    .spacing(40);

    let footer = row![
        button(container(icons::icon(icons::POWER, 18.0, theme::INK)).center(Length::Fill))
            .width(Length::Fixed(52.0))
            .height(Length::Fixed(52.0))
            .padding(0)
            .style(theme::round(false))
            .on_press_maybe((!app.exec.stopping).then(|| Message::VmStop(vm.name.clone(), false))),
        button(theme::label(app.t("exec-force-off"), 14.0, theme::MUTED))
            .padding([0, 2])
            .style(theme::ghost)
            .on_press(Message::VmStop(vm.name.clone(), true)),
        button(theme::label(app.t("exec-back"), 14.0, theme::MUTED))
            .padding([0, 2])
            .style(theme::ghost)
            .on_press(Message::Go(Screen::Painel)),
        iced::widget::space().width(Length::Fill),
        button(theme::label(
            app.t("exec-is-this-normal"),
            13.0,
            theme::MUTED
        ))
        .padding(0)
        .style(theme::ghost)
        .on_press(Message::Go(Screen::Comportamento)),
    ]
    .spacing(16)
    .align_y(iced::Center);

    column![
        hero,
        widgets::risen(r1, meters),
        theme::label(
            app.t("exec-no-reading"),
            12.0,
            theme::with_alpha(theme::DIM, a)
        ),
        iced::widget::space().height(Length::Fill),
        widgets::risen(r2, footer),
    ]
    .spacing(32)
    .height(Length::Fill)
    .into()
}
