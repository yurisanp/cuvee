pub mod bubbles;
pub mod dot;
pub mod icons;
pub mod rail;
pub mod ring;
pub mod spark;

use crate::anim::Rise;
use crate::theme;
use iced::widget::{column, container, rule, space, Space};
use iced::{Element, Length};

/// Wraps an element in the entrance motion: it arrives from 14 px below while fading in.
/// Opacity on arbitrary widgets is not available, so callers fade their own colours with
/// [`theme::with_alpha`] and this adds the rise.
pub fn risen<'a, M: 'a>(rise: Rise, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(iced::Padding {
            top: rise.offset,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .into()
}

pub fn hairline<'a, M: 'a>() -> Element<'a, M> {
    rule::horizontal(1).style(theme::hairline).into()
}

pub fn gap(px: f32) -> Space {
    space().height(Length::Fixed(px))
}

pub fn hgap(px: f32) -> Space {
    space().width(Length::Fixed(px))
}

/// A small caption over a value, as the mockups' "Vídeo / Instalação / Sugerido" columns.
pub fn captioned<'a, M: 'a>(caption: String, value: String, alpha: f32) -> Element<'a, M> {
    column![
        theme::label(caption, 12.0, theme::with_alpha(theme::MUTED, alpha)),
        theme::label(value, 15.0, theme::with_alpha(theme::INK, alpha)),
    ]
    .spacing(6)
    .into()
}
