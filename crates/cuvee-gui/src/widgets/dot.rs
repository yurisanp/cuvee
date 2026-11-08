//! The 8 px state dot: solid and breathing when running, an outline when off.

use crate::theme;
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct Dot {
    pub color: Option<Color>,
    /// 0 → 1 → 0: how far the halo has spread.
    pub breath: f32,
}

impl<M> canvas::Program<M> for Dot {
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
        match self.color {
            Some(color) => {
                if self.breath > 0.0 {
                    let r = 4.0 + 9.0 * self.breath;
                    f.fill(
                        &Path::circle(c, r),
                        theme::with_alpha(color, 0.45 * (1.0 - self.breath)),
                    );
                }
                f.fill(&Path::circle(c, 4.0), color);
            }
            None => f.stroke(
                &Path::circle(c, 3.5),
                Stroke::default().with_color(theme::DIM).with_width(1.0),
            ),
        }
        vec![f.into_geometry()]
    }
}

pub fn dot<'a, M: 'a>(color: Option<Color>, breath: f32) -> Element<'a, M> {
    Canvas::new(Dot { color, breath })
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .into()
}
