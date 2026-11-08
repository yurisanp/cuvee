//! Champagne bubbles: thin golden rings rising and fading, each on its own loop.

use crate::theme;
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};

/// (x as a share of the width, radius, period in seconds, delay in seconds): the mockups' seeds.
const SEEDS: [(f32, f32, f32, f32); 12] = [
    (0.08, 2.0, 7.0, 0.0),
    (0.22, 3.0, 9.0, 1.2),
    (0.38, 1.5, 6.0, 2.4),
    (0.52, 2.5, 8.0, 0.6),
    (0.66, 2.0, 10.0, 3.0),
    (0.78, 3.5, 7.0, 1.8),
    (0.90, 1.5, 9.0, 4.0),
    (0.14, 2.5, 11.0, 3.6),
    (0.30, 2.0, 8.0, 5.0),
    (0.60, 1.5, 6.0, 4.4),
    (0.84, 2.5, 9.0, 2.2),
    (0.46, 3.0, 12.0, 0.3),
];

pub struct Bubbles {
    /// Seconds since the screen opened; the bubbles are a pure function of it.
    pub t: f32,
    pub opacity: f32,
}

impl<M> canvas::Program<M> for Bubbles {
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
        if self.opacity <= 0.0 {
            return vec![f.into_geometry()];
        }
        for (x, r, period, delay) in SEEDS {
            let local = self.t - delay;
            if local < 0.0 {
                continue;
            }
            let p = (local % period) / period;
            // Opacity: in over the first 15 %, then out.
            let a = if p < 0.15 {
                p / 0.15 * 0.8
            } else {
                0.8 * (1.0 - (p - 0.15) / 0.85)
            };
            let scale = 0.5 + 0.5 * p;
            let y = bounds.height - p * bounds.height;
            let path = Path::circle(Point::new(x * bounds.width, y), r * scale);
            f.stroke(
                &path,
                Stroke::default()
                    .with_color(theme::with_alpha(theme::ACCENT, a * self.opacity))
                    .with_width(1.0),
            );
        }
        vec![f.into_geometry()]
    }
}

pub fn bubbles<'a, M: 'a>(t: f32, opacity: f32, width: f32, height: f32) -> Element<'a, M> {
    Canvas::new(Bubbles { t, opacity })
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}
