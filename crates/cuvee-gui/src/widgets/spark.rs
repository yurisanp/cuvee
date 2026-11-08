//! A sparkline: the last N samples of a 0–100 value as a thin polyline.

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct Spark {
    pub samples: Vec<f32>,
    pub color: Color,
}

impl<M> canvas::Program<M> for Spark {
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
        if self.samples.len() >= 2 {
            let n = self.samples.len() as f32 - 1.0;
            let path = Path::new(|b| {
                for (i, v) in self.samples.iter().enumerate() {
                    let x = i as f32 / n * bounds.width;
                    let y =
                        bounds.height - (v.clamp(0.0, 100.0) / 100.0) * (bounds.height - 4.0) - 2.0;
                    if i == 0 {
                        b.move_to(Point::new(x, y));
                    } else {
                        b.line_to(Point::new(x, y));
                    }
                }
            });
            f.stroke(
                &path,
                Stroke::default()
                    .with_color(self.color)
                    .with_width(1.5)
                    .with_line_join(canvas::LineJoin::Round),
            );
        }
        vec![f.into_geometry()]
    }
}

pub fn spark<'a, M: 'a>(samples: Vec<f32>, color: Color) -> Element<'a, M> {
    Canvas::new(Spark { samples, color })
        .width(Length::Fill)
        .height(Length::Fixed(48.0))
        .into()
}
