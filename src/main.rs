mod draw;
mod theme;

use std::io::{Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use crate::draw::Draw;
use crate::theme::WaylandTheme;
use megaui::hash;
use megaui::types::{Point2, Vector2};
use megaui::Ui;
use rusttype::{point, FontCollection, PositionedGlyph, Scale};
use smithay_client_toolkit::keyboard::{
    map_keyboard_auto_with_repeat, Event as KbEvent, KeyRepeatEvent, KeyRepeatKind,
};
use smithay_client_toolkit::reexports::client::protocol::wl_pointer;
use smithay_client_toolkit::reexports::client::protocol::{wl_shm, wl_surface};
use smithay_client_toolkit::reexports::client::{Display, NewProxy};
use smithay_client_toolkit::utils::{DoubleMemPool, MemPool};
use smithay_client_toolkit::window::{ConceptFrame, Event as WEvent, Window};
use smithay_client_toolkit::Environment;
use std::sync::mpsc::{channel, sync_channel};

fn main() {
    let font_data = include_bytes!("../DejaVuSansMono.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
        panic!("error constructing a FontCollection from bytes: {}", e);
    });
    let font = collection
        .into_font() // only succeeds if collection consists of one font
        .unwrap_or_else(|e| {
            panic!("error turning FontCollection into a Font: {}", e);
        });

    // Desired font pixel height
    let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
    let pixel_height = height.ceil() as usize;

    // 2x scale in x direction to counter the aspect ratio of monospace characters.
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    // Glyphs to draw for "RustType". Feel free to try other strings.
    let glyphs: Vec<PositionedGlyph<'_>> = font.layout("RustType", scale, offset).collect();

    // Find the most visually pleasing width to display
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    let (display, mut event_queue) = Display::connect_to_env().unwrap();
    let env = Environment::from_display(&*display, &mut event_queue).unwrap();
    /*
     * Create a buffer with window contents
     */

    let mut dimensions = (320u32, 240u32);

    /*
     * Init wayland objects
     */

    let surface = env
        .compositor
        .create_surface(NewProxy::implement_dummy)
        .unwrap();

    let next_action = Arc::new(Mutex::new(None::<WEvent>));

    let waction = next_action.clone();
    let mut window = Window::<ConceptFrame>::init_from_env(&env, surface, dimensions, move |evt| {
        let mut next_action = waction.lock().unwrap();
        // Keep last event in priority order : Close > Configure > Refresh
        let replace = match (&evt, &*next_action) {
            (_, &None)
            | (_, &Some(WEvent::Refresh))
            | (&WEvent::Configure { .. }, &Some(WEvent::Configure { .. }))
            | (&WEvent::Close, _) => true,
            _ => false,
        };
        if replace {
            *next_action = Some(evt);
        }
    })
    .expect("Failed to create a window !");
    window.set_decorate(true);
    let scaled_bg = [0xFF, 0x22, 0x22, 0x22];
    let vscaled_bg = [0xFF, 0x33, 0x33, 0x33];
    window.set_theme(WaylandTheme {
        primary_active: scaled_bg,
        primary_inactive: vscaled_bg,
        secondary_active: [0xFF, 0xFF, 0xFF, 0xFF],
        secondary_inactive: [0xFF, 0xFF, 0xFF, 0xFF],
        close_button_hovered: [0xFF, 0xFF, 0x00, 0x00],
        close_button: [0xFF, 0x88, 0x00, 0x00],
        close_button_icon_hovered: scaled_bg,
        close_button_icon: [0xFF, 0xff, 0xff, 0xff],
        maximize_button_hovered: [0xFF, 0x00, 0xFF, 0x00],
        maximize_button: [0xFF, 0x00, 0x88, 0x00],
        minimize_button_hovered: [0xFF, 0x00, 0x00, 0xFF],
        minimize_button: [0xFF, 0x00, 0x00, 0x88],
    });

    window.set_title("Perturabo".to_string());

    let mut pools = DoubleMemPool::new(&env.shm, || {}).expect("Failed to create a memory pool !");

    /*
     * Keyboard initialization
     */

    // initialize a seat to retrieve keyboard events
    let seat = env
        .manager
        .instantiate_range(1, 6, NewProxy::implement_dummy)
        .unwrap();

    window.new_seat(&seat);
    let mut ui = Ui::new();
    let main_surface = window.surface().clone();

    enum MouseEvent {
        Move(Point2),
    }

    let (mouse_events_sender, mouse_events_receiver) = sync_channel(1);

    seat.get_pointer(|ptr| {
        ptr.implement_closure(
            move |evt, _| match evt {
                wl_pointer::Event::Enter {
                    surface,
                    surface_x,
                    surface_y,
                    ..
                } => {
                    if main_surface == surface {
                        println!("Pointer entered at ({}, {})", surface_x, surface_y);
                    }
                }
                wl_pointer::Event::Leave { surface, .. } => {
                    if main_surface == surface {
                        println!("Pointer left");
                    }
                }
                wl_pointer::Event::Button { button, state, .. } => {
                    println!("Button {:?} was {:?}", button, state);
                }
                wl_pointer::Event::Motion {
                    surface_x,
                    surface_y,
                    ..
                } => {
                    println!("Pointer motion to ({}, {})", surface_x, surface_y);
                    //                    ui.mouse_move(Point2::new(surface_x as f32, surface_y as f32));
                    mouse_events_sender.send(MouseEvent::Move(Point2::new(
                        surface_x as f32,
                        surface_y as f32,
                    )));
                }
                _ => {}
            },
            (),
        )
    })
    .unwrap();

    map_keyboard_auto_with_repeat(
        &seat,
        KeyRepeatKind::System,
        |event: KbEvent, _| match event {
            KbEvent::Enter { keysyms, .. } => {
                println!("Gained focus while {} keys pressed.", keysyms.len(),);
            }
            KbEvent::Leave { .. } => {
                println!("Lost focus.");
            }
            KbEvent::Key {
                keysym,
                state,
                utf8,
                ..
            } => {
                println!("Key {:?}: {:x}.", state, keysym);
                if let Some(txt) = utf8 {
                    println!(" -> Received text \"{}\".", txt);
                }
            }
            KbEvent::RepeatInfo { rate, delay } => {
                println!(
                    "Received repeat info: start repeating every {}ms after an initial delay of {}ms",
                    rate, delay
                );
            }
            KbEvent::Modifiers { modifiers } => {
                println!("Modifiers changed {:?}", modifiers);
            }
        },
        move |repeat_event: KeyRepeatEvent, _| {
            println!("Repeated key {:x}.", repeat_event.keysym);
            if let Some(txt) = repeat_event.utf8 {
                println!(" -> Received text \"{}\".", txt);
            }
        },
    ).expect("Failed to map keyboard");

    if !env.shell.needs_configure() {
        // initial draw to bootstrap on wl_shell
        if let Some(pool) = pools.pool() {
            redraw(pool, window.surface(), dimensions, &mut ui).expect("Failed to draw")
        }
        window.refresh();
    }

    loop {
        if let Ok(mouse_event) = mouse_events_receiver.try_recv() {
            match mouse_event {
                MouseEvent::Move(point) => {
                    ui.mouse_move(point);
                    println!("mouse event received");
                }
            }
        }
        match next_action.lock().unwrap().take() {
            Some(WEvent::Close) => break,
            Some(WEvent::Refresh) => {
                window.refresh();
                window.surface().commit();
            }
            Some(WEvent::Configure { new_size, states }) => {
                if let Some((w, h)) = new_size {
                    window.resize(w, h);
                    dimensions = (w, h)
                }
                println!("Window states: {:?}", states);
                window.refresh();
                if let Some(pool) = pools.pool() {
                    redraw(pool, window.surface(), dimensions, &mut ui).expect("Failed to draw")
                }
            }
            None => {}
        }

        event_queue.dispatch().unwrap();
    }
}

fn draw_point(buf: &mut [u8], x: u32, y: u32, w: u32) {
    buf[(x * 4 + y * (w * 4)) as usize] = 255;
}

fn redraw(
    pool: &mut MemPool,
    surface: &wl_surface::WlSurface,
    (buf_x, buf_y): (u32, u32),
    ui: &mut Ui,
) -> Result<(), ::std::io::Error> {
    // resize the pool if relevant
    pool.resize((4 * buf_x * buf_y) as usize)
        .expect("Failed to resize the memory pool.");
    // write the contents, a nice color gradient =)
    pool.seek(SeekFrom::Start(0))?;
    let buff: &mut [u8] = pool.mmap();
    let mut draw = Draw {
        width: buf_x,
        height: buf_y,
        buff,
    };
    ui.begin_frame();
    let mut counter = 0;
    megaui::widgets::Window::new(hash!(), Point2::new(50., 50.), Vector2::new(200., 100.)).ui(
        ui,
        |ui| {
            ui.label(Point2::new(20., 20.), &format!("Counter: {}", counter));
            if ui.button(Point2::new(100., 50.), hash!(), "increment") {
                counter += 1;
            }
        },
    );
    ui.draw(&mut draw);
    let new_buffer = pool.buffer(
        0,
        buf_x as i32,
        buf_y as i32,
        4 * buf_x as i32,
        wl_shm::Format::Argb8888,
    );
    surface.attach(Some(&new_buffer), 0, 0);
    surface.commit();
    Ok(())
}
