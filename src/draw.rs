use megaui::types::{Point2, Rect, RectAttr, Vector2};
use megaui::Context;

struct Draw;

impl Context for Draw {
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
        unimplemented!()
    }

    fn clip(&mut self, rect: Option<Rect>) {
        unimplemented!()
    }
}
