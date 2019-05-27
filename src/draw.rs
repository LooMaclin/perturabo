use megaui::types::{Point2, Rect, RectAttr, Vector2};
use megaui::Context;

pub struct Draw<'a> {
    pub width: u32,
    pub height: u32,
    pub buff: &'a mut [u8],
}

impl<'a> Draw<'a> {
    pub fn point(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        let y_offset = y * (self.width * 4);
        let x_offset = x * 4;
        let position = (x_offset + y_offset) as usize;
        self.buff[position] = a;
        self.buff[position + 1] = r;
        self.buff[position + 2] = g;
        self.buff[position + 3] = b;
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
        unimplemented!()
    }

    fn draw_line(&mut self, start: Point2, end: Point2, color: &str) {
        (start.x as u32..=end.x as u32)
            .for_each(|x| self.point(x as u32, start.y as u32, 255, 0, 0, 0));
    }

    fn clip(&mut self, rect: Option<Rect>) {
        unimplemented!()
    }
}
