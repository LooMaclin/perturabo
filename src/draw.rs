use megaui::types::{Color, Point2, Rect, RectAttr, Vector2};
use megaui::Context;

pub struct Draw<'a> {
    pub width: u32,
    pub height: u32,
    pub buff: &'a mut [u8],
}

impl<'a> Draw<'a> {
    pub fn point(&mut self, x: u32, y: u32, color: Color) {
        let y_offset = y * (self.width * 4);
        let x_offset = x * 4;
        let position = (x_offset + y_offset) as usize;
        let (r, g, b, a) = color.to_rgba();
        self.buff[position] = b;
        self.buff[position + 1] = g;
        self.buff[position + 2] = r;
        self.buff[position + 3] = a;
    }
}

impl<'a> Context for Draw<'a> {
    fn draw_label(
        &mut self,
        label: &str,
        position: Point2,
        _: Option<()>,
        _: Option<()>,
        color: Option<&str>,
    ) {
        unimplemented!()
    }

    fn measure_label(&mut self, label: &str, _: Option<()>) -> Vector2 {
        unimplemented!()
    }

    fn draw_rect(&mut self, rect: Rect, attrs: &[RectAttr]) {
        self.draw_line(
            Point2 {
                x: rect.x,
                y: rect.y,
            },
            Point2 {
                x: rect.x + rect.w,
                y: rect.y,
            },
            Color::from_rgba(255, 0, 0, 0),
        );
        self.draw_line(
            Point2 {
                x: rect.x,
                y: rect.y,
            },
            Point2 {
                x: rect.x,
                y: rect.y + rect.h,
            },
            Color::from_rgba(255, 0, 0, 0),
        );
        self.draw_line(
            Point2 {
                x: rect.x + rect.w,
                y: rect.y,
            },
            Point2 {
                x: rect.x + rect.w,
                y: rect.y + rect.h,
            },
            Color::from_rgba(255, 0, 0, 0),
        );
        self.draw_line(
            Point2 {
                x: rect.x,
                y: rect.y + rect.h,
            },
            Point2 {
                x: rect.x + rect.w,
                y: rect.y + rect.h,
            },
            Color::from_rgba(255, 0, 0, 0),
        );
    }

    fn draw_line<T>(&mut self, start: Point2, end: Point2, color: T)
    where
        T: Into<Color>,
    {
        let color = color.into();
        if start.x != end.x {
            for x in start.x as u32..=end.x as u32 {
                self.point(x as u32, start.y as u32, color);
            }
        } else {
            for y in start.y as u32..=end.y as u32 {
                self.point(start.x as u32, y, color);
            }
        }
    }

    fn clip(&mut self, rect: Option<Rect>) {
        unimplemented!()
    }
}
