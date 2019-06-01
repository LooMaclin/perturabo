use itertools::Itertools;
use megaui::types::{Color, Point2, Rect, RectAttr, Vector2};
use megaui::Context;
use rusttype::{point, FontCollection, Scale};

pub struct Draw<'a> {
    pub width: u32,
    pub height: u32,
    pub buff: &'a mut [u8],
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
        let font_data = include_bytes!("../DejaVuSansMono.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
            panic!("error constructing a FontCollection from bytes: {}", e);
        });
        let font = collection
            .into_font() // only succeeds if collection consists of one font
            .unwrap_or_else(|e| {
                panic!("error turning FontCollection into a Font: {}", e);
            });

        for glyph in font.layout(label, Scale::uniform(25.0), point(position.x, position.y)) {
            let pos = glyph.position();
            let size = glyph.scale();
            let bb = glyph.pixel_bounding_box();
            // if no bounding box - we suppose that its invalid character but want it to be draw as empty quad
            let bb = if let Some(bb) = bb {
                bb
            } else {
                rusttype::Rect {
                    min: point(0, 0),
                    max: point((size.x / 2.) as i32, 0),
                }
            };
            glyph.draw(|x, y, v| {
                self.buff[(pos.x as u32 * 4
                    + x * 4
                    + ((bb.min.y + y as i32 + size.y as i32 - pos.y as i32).max(0) as u32)
                        * (self.width * 4)) as usize] = (v * 255.) as u8;
            });
        }
    }

    fn measure_label(&mut self, label: &str, _: Option<()>) -> Vector2 {
        println!("measure label");
        Vector2::new(200., 100.)
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
        println!("clip");
    }
}
