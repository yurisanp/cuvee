//! Painel: the VMs, one per row, and the host's state in one line.

use crate::theme;
use crate::widgets::{self, bubbles, dot, icons};
use crate::{App, Message, Screen};
use cuvee_core::host::Level;
use cuvee_core::i18n::Locale;
use cuvee_core::vm::State;
use iced::widget::{button, column, container, row, stack, Column};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let running = app.vms.iter().filter(|v| v.state == State::Running).count();

    // Host line, top right.
    let host_line: Element<'_, Message> = match app.report.as_deref() {
        Some(r) => {
            let (color, text) = match r.worst() {
                Level::Ok => (theme::GOOD, app.t("painel-host-ok")),
                Level::Warn => {
                    let n = r.checks.iter().filter(|c| c.level != Level::Ok).count();
                    (
                        theme::WARN,
                        app.locale
                            .t_args("painel-host-warn", &[("n", Locale::num(n as f64))]),
                    )
                }
                Level::Bad => (theme::BAD, app.t("painel-host-bad")),
            };
            let r0 = clock.rise(t0, 0);
            button(
                row![
                    container(dot::dot(Some(theme::with_alpha(color, r0.opacity)), 0.0))
                        .width(Length::Fixed(16.0))
                        .center_y(Length::Fixed(28.0)),
                    theme::label(text, 13.0, theme::with_alpha(theme::MUTED, r0.opacity)),
                ]
                .spacing(6)
                .align_y(iced::Center),
            )
            .padding(0)
            .style(theme::ghost)
            .on_press(Message::Go(Screen::Diagnostico))
            .into()
        }
        None => widgets::gap(28.0).into(),
    };

    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);
    let title = theme::serif(
        app.locale
            .t_args("painel-title", &[("n", Locale::num(app.vms.len() as f64))]),
        72.0,
        theme::with_alpha(theme::INK, r1.opacity),
    );
    let subtitle_text = if app.vms.is_empty() {
        app.t("painel-empty-hint")
    } else {
        app.locale
            .t_args("painel-running", &[("n", Locale::num(running as f64))])
    };
    let subtitle = theme::label(
        subtitle_text,
        18.0,
        theme::with_alpha(theme::MUTED, r2.opacity),
    );

    let hero = stack![
        container(bubbles::bubbles(clock.seconds(t0), 1.0, 320.0, 240.0))
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        column![widgets::risen(r1, title), widgets::risen(r2, subtitle)]
            .spacing(14)
            .padding([24, 0]),
    ];

    let mut list: Column<'_, Message> = column![widgets::hairline()];
    for (i, vm) in app.vms.iter().enumerate() {
        let r = clock.rise(t0, 3 + i);
        let a = r.opacity;
        let running = vm.state == State::Running;
        let state_dot = if running {
            dot::dot(
                Some(theme::with_alpha(theme::GOOD, a)),
                clock.breathe(t0, 2.4),
            )
        } else {
            dot::dot(None, 0.0)
        };
        let subtitle = if running {
            app.t("vm-accelerated")
        } else {
            app.t("vm-not-accelerated")
        };
        let action: Element<'_, Message> = if running {
            button(
                container(icons::icon(icons::ARROW_UP_RIGHT, 16.0, theme::INK))
                    .center(Length::Fill),
            )
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .padding(0)
            .style(theme::round(true))
            .on_press(Message::Go(Screen::Execucao(i)))
            .into()
        } else {
            button(container(icons::icon(icons::PLAY, 14.0, theme::INK)).center(Length::Fill))
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0))
                .padding(0)
                .style(theme::round(true))
                .on_press(Message::VmStart(vm.name.clone()))
                .into()
        };
        let clock_text = vm
            .pid
            .and_then(cuvee_core::vm::uptime)
            .map(|d| format!("{:02}:{:02}", d.as_secs() / 3600, d.as_secs() / 60 % 60))
            .unwrap_or_default();
        let row_body = row![
            state_dot,
            column![
                theme::serif(vm.name.clone(), 28.0, theme::with_alpha(theme::INK, a))
                    .font(theme::SERIF_REGULAR),
                theme::label(subtitle, 13.0, theme::with_alpha(theme::MUTED, a)),
            ]
            .spacing(4)
            .width(Length::Fill),
            theme::mono(clock_text, 13.0, theme::with_alpha(theme::MUTED, a)),
            action,
        ]
        .spacing(24)
        .align_y(iced::Center)
        .padding([22, 16]);
        let row_button = button(row_body)
            .padding(0)
            .style(theme::row_hover)
            .on_press(Message::Go(if running {
                Screen::Execucao(i)
            } else {
                Screen::Painel
            }));
        list = list
            .push(widgets::risen(r, row_button))
            .push(widgets::hairline());
    }

    let rf = clock.rise(t0, 4 + app.vms.len());
    let footer = row![
        button(theme::label(
            app.t("painel-new"),
            15.0,
            theme::with_alpha(theme::INK, rf.opacity)
        ))
        .padding([0, 2])
        .style(theme::ghost_bright)
        .on_press(Message::Go(Screen::Nova)),
        iced::widget::space().width(Length::Fill),
        row![
            theme::label(
                language_name(app),
                13.0,
                theme::with_alpha(theme::MUTED, rf.opacity)
            ),
            theme::label(" · ", 13.0, theme::with_alpha(theme::MUTED, rf.opacity)),
            button(theme::label(
                app.t("painel-change-language"),
                13.0,
                theme::with_alpha(theme::MUTED, rf.opacity)
            ))
            .padding(0)
            .style(theme::ghost)
            .on_press(Message::Go(Screen::Ajustes)),
        ]
        .align_y(iced::Center),
    ]
    .align_y(iced::Center);

    column![
        container(host_line)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        hero,
        list,
        iced::widget::space().height(Length::Fill),
        widgets::risen(rf, footer),
    ]
    .spacing(24)
    .height(Length::Fill)
    .into()
}

fn language_name(app: &App) -> String {
    cuvee_core::i18n::LANGUAGES
        .iter()
        .find(|l| l.tag == app.locale.tag())
        .map(|l| l.native_name.to_string())
        .unwrap_or_default()
}
