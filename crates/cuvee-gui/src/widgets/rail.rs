//! The icon-only navigation rail on the left.

use super::icons;
use crate::theme;
use crate::{Message, Screen};
use iced::widget::{button, column, container, row, rule, tooltip};
use iced::{Element, Length};

const ITEMS: [(Screen, &str, &str); 5] = [
    (Screen::Painel, icons::GRID, "nav-painel"),
    (Screen::Nova, icons::PLUS, "nav-nova"),
    (Screen::Diagnostico, icons::PULSE, "nav-diagnostico"),
    (Screen::Comportamento, icons::BOOK, "nav-comportamento"),
    (Screen::Ajustes, icons::SLIDERS, "nav-ajustes"),
];

pub fn view<'a>(current: Screen, t: &dyn Fn(&str) -> String) -> Element<'a, Message> {
    let mut items =
        column![container(icons::icon(icons::LOGO, 26.0, theme::ACCENT)).padding([0, 0]),]
            .spacing(8)
            .align_x(iced::Center);
    items = items.push(super::gap(16.0));
    for (screen, src, key) in ITEMS {
        let active = screen.same_section(current);
        let color = if active { theme::INK } else { theme::MUTED };
        let b = button(container(icons::icon(src, 20.0, color)).center(Length::Fill))
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(44.0))
            .padding(0)
            .style(theme::rail_slot(active))
            .on_press(Message::Go(screen));
        items = items.push(
            tooltip(
                b,
                theme::label(t(key), 12.0, theme::INK),
                tooltip::Position::Right,
            )
            .gap(8)
            .style(theme::code_block),
        );
    }
    row![
        container(items)
            .width(Length::Fixed(71.0))
            .height(Length::Fill)
            .padding([28, 0]),
        rule::vertical(1).style(theme::hairline),
    ]
    .height(Length::Fill)
    .into()
}
