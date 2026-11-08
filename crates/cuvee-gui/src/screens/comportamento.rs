//! Comportamento: what to expect from each family of macOS versions.

use crate::theme;
use crate::widgets::{self, dot};
use crate::{App, Message};
use cuvee_core::macos::{Group, Rating};
use iced::widget::{button, column, container, row, rule, Column, Row};
use iced::{Element, Length};
use std::time::Duration;

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let r0 = clock.rise(t0, 0);
    let r1 = clock.rise(t0, 1);

    let mut tabs = Row::new().spacing(32);
    for g in Group::ALL {
        let on = g == app.group;
        let grow = if on {
            clock.progress(
                app.group_switched_at,
                Duration::ZERO,
                Duration::from_millis(450),
            )
        } else {
            0.0
        };
        let color = if on { theme::INK } else { theme::MUTED };
        let item = column![
            theme::label(
                app.t(g.label_key()),
                15.0,
                theme::with_alpha(color, r1.opacity)
            ),
            container(rule::horizontal(2).style(theme::hairline_colored(theme::ACCENT)))
                .padding(iced::Padding {
                    top: 6.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0
                })
                .width(Length::Fixed(grow * 100.0))
                .height(Length::Fixed(8.0)),
        ];
        tabs = tabs.push(
            button(item)
                .padding([8, 0])
                .style(theme::bare)
                .on_press(Message::PickGroup(g)),
        );
    }

    let mut rows: Column<'_, Message> = column![widgets::hairline()];
    for (i, b) in app.group.behaviors().iter().enumerate() {
        let p = clock.progress(
            app.group_switched_at,
            Duration::from_millis(60 * i as u64),
            Duration::from_millis(550),
        );
        let a = p;
        let color = match b.rating {
            Rating::Good => theme::GOOD,
            Rating::Limited => theme::WARN,
            Rating::Bad => theme::BAD,
        };
        let line = row![
            container(theme::label(
                app.t(b.aspect.label_key()),
                13.0,
                theme::with_alpha(theme::MUTED, a)
            ))
            .width(Length::Fixed(200.0)),
            dot::dot(Some(theme::with_alpha(color, a)), 0.0),
            theme::label(app.t(b.text_key), 16.0, theme::with_alpha(theme::INK, a)),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .padding([18, 12]);
        // Slide in from the left by 12 px as it fades.
        let slide = container(line).padding(iced::Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 12.0 * (1.0 - p),
        });
        rows = rows.push(slide).push(widgets::hairline());
    }

    let r3 = clock.rise(t0, 3);
    let a = r3.opacity;
    let legend = row![
        column![
            theme::label(
                app.t("comp-normal"),
                12.0,
                theme::with_alpha(theme::MUTED, a)
            ),
            theme::label(
                app.t("comp-normal-text"),
                14.0,
                theme::with_alpha(theme::SOFT, a)
            )
            .line_height(1.6),
        ]
        .spacing(8)
        .width(Length::Fill),
        column![
            theme::label(
                app.t("comp-ask-help"),
                12.0,
                theme::with_alpha(theme::MUTED, a)
            ),
            theme::label(
                app.t("comp-ask-help-text"),
                14.0,
                theme::with_alpha(theme::SOFT, a)
            )
            .line_height(1.6),
        ]
        .spacing(8)
        .width(Length::Fill),
        column![
            theme::label(
                app.t("comp-legend"),
                12.0,
                theme::with_alpha(theme::MUTED, a)
            ),
            row![
                dot::dot(Some(theme::with_alpha(theme::GOOD, a)), 0.0),
                theme::label(
                    app.t("comp-legend-good"),
                    14.0,
                    theme::with_alpha(theme::SOFT, a)
                ),
                dot::dot(Some(theme::with_alpha(theme::WARN, a)), 0.0),
                theme::label(
                    app.t("comp-legend-limited"),
                    14.0,
                    theme::with_alpha(theme::SOFT, a)
                ),
                dot::dot(Some(theme::with_alpha(theme::BAD, a)), 0.0),
                theme::label(
                    app.t("comp-legend-bad"),
                    14.0,
                    theme::with_alpha(theme::SOFT, a)
                ),
            ]
            .align_y(iced::Center),
            theme::label(
                app.t("comp-validate"),
                12.0,
                theme::with_alpha(theme::DIM, a)
            ),
        ]
        .spacing(8)
        .width(Length::Fill),
    ]
    .spacing(32);

    column![
        widgets::risen(
            r0,
            theme::serif(
                app.t("comp-title"),
                56.0,
                theme::with_alpha(theme::INK, r0.opacity)
            )
        ),
        widgets::risen(r1, tabs),
        rows,
        widgets::risen(r3, legend),
    ]
    .spacing(36)
    .max_width(1040)
    .into()
}
