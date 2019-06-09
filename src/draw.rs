use megaui::types::{Color, Point2, Rect, Vector2};
use megaui::Context;
use rusttype::{point, FontCollection, Scale};

pub struct Draw<'a> {
    pub width: u32,
    pub height: u32,
    pub buff: &'a mut [u8],
    pub draw_rect: Option<Rect>,
}

impl<'a> Draw<'a> {
    pub fn fill<T>(&mut self, color: T)
    where
        T: Into<Color>,
    {
        let (r, g, b, a) = color.into().to_rgba();
        self.buff.chunks_mut(4).for_each(|rgba| {
            rgba[0] = b;
            rgba[1] = g;
            rgba[2] = r;
            rgba[3] = a;
        });
    }
    pub fn point(&mut self, x: u32, y: u32, color: Color) {
        let visible = self
            .draw_rect
            .map(|draw_rect| {
                x >= draw_rect.x as u32
                    && y >= draw_rect.y as u32
                    && x <= draw_rect.x as u32 + draw_rect.w as u32
                    && y <= draw_rect.y as u32 + draw_rect.h as u32
            })
            .unwrap_or(true);
        if visible {
            let y_offset = y * (self.width * 4);
            let x_offset = x * 4;
            let position = (x_offset + y_offset) as usize;
            let (r, g, b, a) = color.to_rgba();
            self.buff[position] =
                (self.buff[position] as f32 * (1. - color.a) + color.a * b as f32) as u8;
            self.buff[position + 1] =
                (self.buff[position + 1] as f32 * (1. - color.a) + color.a * g as f32) as u8;
            self.buff[position + 2] =
                (self.buff[position + 2] as f32 * (1. - color.a) + color.a * r as f32) as u8;
            self.buff[position + 3] = 255;
        }
    }
}

impl<'a> Context for Draw<'a> {
    fn draw_label<T: Into<Color>>(
        &mut self,
        label: &str,
        position: Point2,
        _: Option<()>,
        _: Option<()>,
        color: Option<T>,
    ) {
        let color = color.unwrap().into();
        let font_data = include_bytes!("../DejaVuSansMono.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
            panic!("error constructing a FontCollection from bytes: {}", e);
        });
        let font = collection.into_font().unwrap_or_else(|e| {
            panic!("error turning FontCollection into a Font: {}", e);
        });

        for glyph in font.layout(label, Scale::uniform(25.0), point(position.x, position.y)) {
            let pos = glyph.position();
            let size = glyph.scale();
            let bb = glyph.pixel_bounding_box();
            let bb = if let Some(bb) = bb {
                bb
            } else {
                rusttype::Rect {
                    min: point(0, 0),
                    max: point((size.x / 2.) as i32, 0),
                }
            };

            let wtf = font.v_metrics(Scale::uniform(20.0));
            glyph.draw(|x, y, v| {
                let y_offset = bb.min.y as i32 + y as i32 + wtf.ascent as i32;
                if y_offset > 0 {
                    self.point(
                        x + pos.x as u32,
                        y_offset as u32,
                        Color {
                            r: v * color.r,
                            a: v * color.a,
                            b: v * color.b,
                            g: v * color.g,
                        },
                    );
                }
            });
        }
    }

    fn measure_label(&mut self, _label: &str, _: Option<()>) -> Vector2 {
        println!("measure label");
        Vector2::new(200., 100.)
    }

    fn draw_rect<S, T>(&mut self, rect: Rect, stroke: S, fill: T)
    where
        S: Into<Option<Color>>,
        T: Into<Option<Color>>,
    {
        if let Some(stroke) = stroke.into() {
            self.draw_line(
                Point2 {
                    x: rect.x,
                    y: rect.y,
                },
                Point2 {
                    x: rect.x + rect.w,
                    y: rect.y,
                },
                stroke,
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
                stroke,
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
                stroke,
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
                stroke,
            );
        }
        if let Some(fill) = fill.into() {
            for x in rect.x as u32 + 1..rect.x as u32 + rect.w as u32 - 1 {
                for y in rect.y as u32 + 1..rect.y as u32 + rect.h as u32 - 1 {
                    self.point(x, y, fill);
                }
            }
        }
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
        self.draw_rect = rect;
    }
}
