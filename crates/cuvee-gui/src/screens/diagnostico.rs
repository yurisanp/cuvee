//! Diagnóstico: the ring, the headline, the fix button and the nine checks.

use crate::theme;
use crate::widgets::{self, icons, ring};
use crate::{App, Message};
use cuvee_core::host::{CheckId, Level};
use cuvee_core::i18n::Locale;
use iced::widget::{button, column, container, row, Column, Row};
use iced::{Element, Length};
use std::time::Duration;

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let total = CheckId::ALL.len();

    let Some(report) = app.report.as_deref() else {
        return container(theme::label(app.t("diag-rerun"), 14.0, theme::MUTED)).into();
    };

    let ok = report.ok_count();
    let worst = report.worst();
    let color = match worst {
        Level::Ok => theme::GOOD,
        Level::Warn => theme::ACCENT,
        Level::Bad => theme::BAD,
    };
    let draw = clock.progress(t0, Duration::from_millis(200), Duration::from_millis(1600));
    let ring_el = ring::ring::<Message>(
        ring::Ring {
            progress: draw * ok as f32 / total as f32,
            color,
            glow: clock.breathe(t0, 3.5),
            count: ok.to_string(),
            caption: app
                .locale
                .t_args("diag-of", &[("total", Locale::num(total as f64))]),
        },
        200.0,
    );

    let problems = report
        .checks
        .iter()
        .filter(|c| c.level != Level::Ok)
        .count();
    let headline = match (worst, problems) {
        (Level::Ok, _) => app.t("diag-all-ok"),
        (Level::Bad, _) => app.t("diag-bad"),
        (Level::Warn, 1) => app.t("diag-one-warn"),
        (Level::Warn, n) => app
            .locale
            .t_args("diag-many-warn", &[("n", Locale::num(n as f64))]),
    };
    let first = report.first_problem();
    let sub = match first {
        Some(c) => c.advice_key.map(|k| app.t(k)).unwrap_or_default(),
        None => app.t("diag-all-ok-sub"),
    };

    let r = clock.rise(t0, 3);
    let mut right = column![
        theme::serif(headline, 52.0, theme::with_alpha(theme::INK, r.opacity)),
        theme::label(sub, 16.0, theme::with_alpha(theme::SOFT, r.opacity)).line_height(1.6),
    ]
    .spacing(18)
    .max_width(520);

    if let Some(c) = first {
        let mut actions = Row::new().spacing(20).align_y(iced::Center);
        if c.fix_command.is_some() {
            let label = if app.diag_fixing {
                "…".to_string()
            } else {
                app.t("diag-fix")
            };
            actions = actions.push(
                button(theme::label(label, 15.0, theme::BG).font(theme::SANS_MEDIUM))
                    .padding([14, 26])
                    .style(theme::cta)
                    .on_press_maybe((!app.diag_fixing).then_some(Message::DiagFix)),
            );
            actions = actions.push(
                button(theme::label(
                    app.t(if app.diag_show_cmd {
                        "diag-hide-command"
                    } else {
                        "diag-show-command"
                    }),
                    14.0,
                    theme::MUTED,
                ))
                .padding([0, 2])
                .style(theme::ghost)
                .on_press(Message::DiagToggleCmd),
            );
        }
        actions = actions.push(
            button(theme::label(app.t("diag-rerun"), 14.0, theme::MUTED))
                .padding([0, 2])
                .style(theme::ghost)
                .on_press(Message::Refresh),
        );
        right = right.push(actions);
        if app.diag_show_cmd {
            if let Some(cmd) = &c.fix_command {
                right = right.push(
                    column![
                        container(theme::mono(format!("$ {cmd}"), 13.0, theme::SOFT))
                            .padding([10, 14])
                            .style(theme::code_block),
                        theme::label(app.t("diag-sudo-note"), 12.0, theme::MUTED),
                    ]
                    .spacing(8),
                );
            }
        }
    }

    let top = row![ring_el, widgets::risen(r, right)]
        .spacing(56)
        .align_y(iced::Center);

    // The checks, two columns.
    let mut cols: [Vec<Element<'_, Message>>; 2] = [Vec::new(), Vec::new()];
    for (i, check) in report.checks.iter().enumerate() {
        let ri = clock.rise(t0, 5 + i);
        let a = ri.opacity;
        let (icon, icon_color, text_color, status) = match check.level {
            Level::Ok => (icons::CHECK, theme::GOOD, theme::SOFT, app.t("diag-ok")),
            Level::Warn => (
                icons::PULSE,
                theme::WARN,
                theme::WARN,
                app.t("diag-needs-fix"),
            ),
            Level::Bad => (
                icons::PULSE,
                theme::BAD,
                theme::BAD,
                app.t("diag-needs-fix"),
            ),
        };
        let line = row![
            icons::icon(icon, 18.0, theme::with_alpha(icon_color, a)),
            theme::label(
                app.t(check.id.title_key()),
                14.0,
                theme::with_alpha(text_color, a)
            ),
            iced::widget::space().width(Length::Fill),
            theme::label(status, 12.0, theme::with_alpha(theme::DIM, a)),
        ]
        .spacing(12)
        .align_y(iced::Center)
        .height(Length::Fixed(44.0));
        cols[i % 2].push(widgets::risen(ri, line));
    }
    let [c0, c1] = cols;
    let checks = row![
        Column::with_children(c0).spacing(4).width(Length::Fill),
        Column::with_children(c1).spacing(4).width(Length::Fill)
    ]
    .spacing(40);

    column![top, widgets::hairline(), container(checks).padding([28, 0])]
        .spacing(28)
        .into()
}
