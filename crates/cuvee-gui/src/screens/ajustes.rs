//! Ajustes: language (a sliding pill), animations, and about.

use crate::theme;
use crate::widgets;
use crate::{App, Message};
use cuvee_core::i18n::{Locale, LANGUAGES};
use iced::widget::{button, column, container, row, Row};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let r0 = clock.rise(t0, 0);
    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);
    let r3 = clock.rise(t0, 3);

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

    column![
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
        widgets::risen(
            r3,
            section(
                app.t("ajustes-about"),
                theme::label(
                    app.locale.t_args(
                        "ajustes-about-text",
                        &[("version", Locale::str(env!("CARGO_PKG_VERSION")))]
                    ),
                    14.0,
                    theme::with_alpha(theme::SOFT, r3.opacity)
                )
                .into(),
                r3.opacity
            )
        ),
    ]
    .spacing(32)
    .max_width(760)
    .width(Length::Fill)
    .into()
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
