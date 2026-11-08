//! Line icons, as in the mockups: 24-unit viewBox, stroked with the current colour.

use crate::theme;
use iced::widget::{svg, Svg};
use iced::{Color, Length};

const fn wrap(body: &'static str) -> &'static str {
    body
}

macro_rules! icon {
    ($name:ident, $body:expr) => {
        pub const $name: &str = concat!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#fff' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'>",
            $body,
            "</svg>"
        );
    };
}

icon!(GRID, "<rect x='4' y='4' width='6' height='6' rx='1.5'/><rect x='14' y='4' width='6' height='6' rx='1.5'/><rect x='4' y='14' width='6' height='6' rx='1.5'/><rect x='14' y='14' width='6' height='6' rx='1.5'/>");
icon!(PLUS, "<path d='M12 5v14M5 12h14'/>");
icon!(PULSE, "<path d='M3 12h4l2-6 4 12 2-6h6'/>");
icon!(BOOK, "<path d='M4 5h6a2 2 0 0 1 2 2v13a2 2 0 0 0-2-2H4V5ZM20 5h-6a2 2 0 0 0-2 2v13a2 2 0 0 1 2-2h6V5Z'/>");
icon!(SLIDERS, "<path d='M4 7h10M18 7h2M4 17h4M12 17h8'/><circle cx='16' cy='7' r='2'/><circle cx='10' cy='17' r='2'/>");
icon!(ARROW_UP_RIGHT, "<path d='M7 17 17 7M9 7h8v8'/>");
icon!(
    PLAY,
    "<path d='M7 4.5v15l13-7.5-13-7.5Z' fill='#fff' stroke='none'/>"
);
icon!(POWER, "<path d='M12 3v8M6.3 6.3a8 8 0 1 0 11.4 0'/>");
icon!(CHECK, "<path d='M5 12.5 9.5 17 19 7.5'/>");

pub const LOGO: &str = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 28 28' fill='none' stroke='#fff' stroke-width='1.5' stroke-linecap='round'><path d='M9 3h10l-1 9a4 4 0 0 1-8 0L9 3Z'/><path d='M14 16v7M10 25h8'/></svg>";

pub fn icon<'a>(src: &'static str, size: f32, color: Color) -> Svg<'a> {
    svg(svg::Handle::from_memory(wrap(src).as_bytes()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(theme::icon_color(color))
}
