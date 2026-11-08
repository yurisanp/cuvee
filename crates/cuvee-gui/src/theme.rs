//! The look: the v2 "minimalista" design tokens, fonts and the few widget styles the app uses.

use iced::border::Radius;
use iced::font::{Family, Weight};
use iced::widget::{button, container, rule, svg, text};
use iced::{color, Border, Color, Font, Shadow, Theme, Vector};

pub const BG: Color = color!(0x0B0A0E);
pub const INK: Color = color!(0xF4F1EA);
pub const SOFT: Color = color!(0xB8B4C0);
pub const MUTED: Color = color!(0x8E8A97);
pub const DIM: Color = color!(0x6E6A78);
pub const LINE: Color = color!(0x16141B);
pub const LINE_STRONG: Color = color!(0x2A2732);
pub const HOVER_BG: Color = color!(0x121016);
pub const RAIL_ACTIVE: Color = color!(0x16141B);
pub const ACCENT: Color = color!(0xE6C77F);
pub const ACCENT_HOVER: Color = color!(0xF6E3B4);
pub const GOOD: Color = color!(0x6FCF97);
pub const WARN: Color = color!(0xF2B45A);
pub const BAD: Color = color!(0xF07A7A);
pub const INFO: Color = color!(0x8AB4F8);

pub const FRAUNCES: &[u8] = include_bytes!("../assets/fonts/Fraunces.ttf");
pub const INTER: &[u8] = include_bytes!("../assets/fonts/Inter.ttf");
pub const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono.ttf");

pub const SERIF: Font = Font {
    family: Family::Name("Fraunces"),
    weight: Weight::Light,
    ..Font::DEFAULT
};
pub const SERIF_REGULAR: Font = Font {
    family: Family::Name("Fraunces"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};
pub const SANS: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};
pub const SANS_MEDIUM: Font = Font {
    family: Family::Name("Inter"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
pub const MONO: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0) * c.a,
        ..c
    }
}

pub fn theme() -> Theme {
    Theme::custom(
        "Cuvée",
        iced::theme::Palette {
            background: BG,
            text: INK,
            primary: ACCENT,
            success: GOOD,
            warning: WARN,
            danger: BAD,
        },
    )
}

pub fn app_style(_: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: BG,
        text_color: INK,
    }
}

/// A text widget in the app's font and colour.
pub fn label<'a>(s: impl text::IntoFragment<'a>, size: f32, color: Color) -> text::Text<'a> {
    text(s).size(size).font(SANS).color(color)
}

pub fn serif<'a>(s: impl text::IntoFragment<'a>, size: f32, color: Color) -> text::Text<'a> {
    text(s).size(size).font(SERIF).color(color)
}

pub fn mono<'a>(s: impl text::IntoFragment<'a>, size: f32, color: Color) -> text::Text<'a> {
    text(s).size(size).font(MONO).color(color)
}

const NO_BORDER: Border = Border {
    color: Color::TRANSPARENT,
    width: 0.0,
    radius: Radius {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    },
};

/// Text-only button: muted, brightens on hover.
pub fn ghost(_: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => INK,
        button::Status::Disabled => DIM,
        button::Status::Active => MUTED,
    };
    button::Style {
        background: None,
        text_color,
        border: NO_BORDER,
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Text-only button that is already bright (the "+ Nova máquina" link).
pub fn ghost_bright(_: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_HOVER,
        _ => INK,
    };
    button::Style {
        background: None,
        text_color,
        border: NO_BORDER,
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The champagne pill.
pub fn cta(_: &Theme, status: button::Status) -> button::Style {
    let (bg, shadow) = match status {
        button::Status::Hovered | button::Status::Pressed => (
            ACCENT_HOVER,
            Shadow {
                color: with_alpha(ACCENT, 0.45),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 30.0,
            },
        ),
        _ => (ACCENT, Shadow::default()),
    };
    button::Style {
        background: Some(bg.into()),
        text_color: BG,
        border: Border {
            radius: Radius::new(999.0),
            ..NO_BORDER
        },
        shadow,
        snap: false,
    }
}

/// A round outlined icon button; fills champagne on hover when `accent_hover`.
pub fn round(accent_hover: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (bg, fg, border) = match (hovered, accent_hover) {
            (true, true) => (Some(ACCENT.into()), BG, ACCENT),
            (true, false) => (None, INK, INK),
            _ => (None, INK, LINE_STRONG),
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                color: border,
                width: 1.0,
                radius: Radius::new(999.0),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A rail icon slot: a soft square when active or hovered.
pub fn rail_slot(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: (active || hovered).then(|| RAIL_ACTIVE.into()),
            text_color: if active || hovered { INK } else { MUTED },
            border: Border {
                radius: Radius::new(12.0),
                ..NO_BORDER
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A list row: tinted on hover.
pub fn row_hover(_: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: hovered.then(|| HOVER_BG.into()),
        text_color: INK,
        border: Border {
            radius: Radius::new(14.0),
            ..NO_BORDER
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A plain invisible button (used for text tabs and version numbers).
pub fn bare(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: INK,
        border: NO_BORDER,
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn hairline(_: &Theme) -> rule::Style {
    rule::Style {
        color: LINE,
        radius: Radius::new(0.0),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

pub fn hairline_colored(c: Color) -> impl Fn(&Theme) -> rule::Style {
    move |_| rule::Style {
        color: c,
        radius: Radius::new(2.0),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

pub fn code_block(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(SOFT),
        background: Some(HOVER_BG.into()),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: Radius::new(10.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn tinted(c: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: None,
        background: Some(c.into()),
        border: Border {
            radius: Radius::new(999.0),
            ..NO_BORDER
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn icon_color(c: Color) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |_, _| svg::Style { color: Some(c) }
}
