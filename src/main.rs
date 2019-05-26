mod theme;

use std::cmp::min;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

use byteorder::{NativeEndian, WriteBytesExt};

use crate::theme::WaylandTheme;
use smithay_client_toolkit::keyboard::{
    map_keyboard_auto_with_repeat, Event as KbEvent, KeyRepeatEvent, KeyRepeatKind,
};
use smithay_client_toolkit::reexports::client::protocol::{wl_shm, wl_surface};
use smithay_client_toolkit::reexports::client::{Display, NewProxy};
use smithay_client_toolkit::utils::{DoubleMemPool, MemPool};
use smithay_client_toolkit::window::{ConceptFrame, Event as WEvent, Window};
use smithay_client_toolkit::Environment;

fn main() {
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

    map_keyboard_auto_with_repeat(
        &seat,
        KeyRepeatKind::System,
        move |event: KbEvent, _| match event {
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
            redraw(pool, window.surface(), dimensions).expect("Failed to draw")
        }
        window.refresh();
    }

    loop {
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
                    redraw(pool, window.surface(), dimensions).expect("Failed to draw")
                }
            }
            None => {}
        }

        event_queue.dispatch().unwrap();
    }
}

fn redraw(
    pool: &mut MemPool,
    surface: &wl_surface::WlSurface,
    (buf_x, buf_y): (u32, u32),
) -> Result<(), ::std::io::Error> {
    // resize the pool if relevant
    pool.resize((4 * buf_x * buf_y) as usize)
        .expect("Failed to resize the memory pool.");
    // write the contents, a nice color gradient =)
    pool.seek(SeekFrom::Start(0))?;
    {
        let mut writer = BufWriter::new(&mut *pool);
        for i in 0..(buf_x * buf_y) {
            let x = (i % buf_x) as u32;
            let y = (i / buf_x) as u32;
            let r: u32 = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g: u32 = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b: u32 = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
            writer.write_u32::<NativeEndian>((0xFF << 24) + (r << 16) + (g << 8) + b)?;
        }
        writer.flush()?;
    }
    // get a buffer and attach it
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
