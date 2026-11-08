//! The diagnostic ring: a 4 px arc that draws itself to the share of checks that pass, with a
//! soft glow behind it, and the count in the middle.

use crate::theme;
use iced::mouse;
use iced::widget::canvas::{self, path, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Pixels, Point, Radians, Rectangle, Renderer, Theme};

pub struct Ring {
    /// Share of the ring drawn, 0 → 1 (already eased by the caller).
    pub progress: f32,
    pub color: Color,
    /// 0 → 1 → 0 glow pulse.
    pub glow: f32,
    pub count: String,
    pub caption: String,
}

impl<M> canvas::Program<M> for Ring {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        let c = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let r = bounds.width.min(bounds.height) / 2.0 - 20.0;

        // Glow: a few concentric discs stand in for a blur.
        let glow_a = 0.10 + 0.14 * self.glow;
        for i in 1..=5 {
            let k = i as f32 / 5.0;
            f.fill(
                &Path::circle(c, r * (0.55 + 0.45 * k)),
                theme::with_alpha(self.color, glow_a * (1.0 - k) * 0.5),
            );
        }

        f.stroke(
            &Path::circle(c, r),
            Stroke::default().with_color(theme::LINE).with_width(4.0),
        );

        if self.progress > 0.0 {
            let start = -std::f32::consts::FRAC_PI_2;
            let end = start + std::f32::consts::TAU * self.progress.min(1.0);
            let arc = Path::new(|b| {
                b.arc(path::Arc {
                    center: c,
                    radius: r,
                    start_angle: Radians(start),
                    end_angle: Radians(end),
                });
            });
            f.stroke(
                &arc,
                Stroke::default()
                    .with_color(self.color)
                    .with_width(4.0)
                    .with_line_cap(canvas::LineCap::Round),
            );
        }

        f.fill_text(Text {
            content: self.count.clone(),
            position: Point::new(c.x, c.y - 4.0),
            color: theme::INK,
            size: Pixels(52.0),
            font: theme::SERIF,
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });
        f.fill_text(Text {
            content: self.caption.clone(),
            position: Point::new(c.x, c.y + 30.0),
            color: theme::MUTED,
            size: Pixels(12.0),
            font: theme::SANS,
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });
        vec![f.into_geometry()]
    }
}

pub fn ring<'a, M: 'a>(ring: Ring, size: f32) -> Element<'a, M> {
    Canvas::new(ring)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
