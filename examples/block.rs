#![no_std]
extern crate alloc;

#[macro_use]
extern crate playdate_sys as sys;
extern crate playdate_controls as controls;
extern crate playdate_display as display;
extern crate playdate_graphics as gfx;
extern crate playdate_system as system;

use core::ffi::*;
use core::ptr::NonNull;

use display::Display;

use sys::EventLoopCtrl;
use sys::ffi::*;
use system::prelude::*;

use controls::api::Cache;
use controls::buttons::IterSingleButtons;
use controls::buttons::PDButtonsExt;
use controls::buttons::PDButtonsIter;
use controls::peripherals::Buttons;
use controls::peripherals::Crank;

use gfx::Graphics;
use gfx::bitmap::Bitmap;
use gfx::color::*;
use gfx::text::StringEncoding;
use gfx::text::StringEncodingExt;

const CENTER_X: u32 = LCD_COLUMNS / 2;
const CENTER_Y: u32 = LCD_ROWS / 2;
const TEXT_HEIGHT: u32 = 16;

/// App state
struct State {
    image: Bitmap,
    crank: Crank<Cache>,
    buttons: Buttons<Cache>,
    pos: Point<i32>,
}

/// 2D point
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    const fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}

impl State {
    fn new() -> Self {
        let image = Bitmap::new(100, 100, Color::BLACK).unwrap();
        let crank = Crank::Cached();
        let buttons = Buttons::Cached();
        let pos = Point::new(CENTER_X as i32, CENTER_Y as i32);
        Self {
            image,
            crank,
            buttons,
            pos,
        }
    }

    /// Event handler
    fn event(&'static mut self, event: SystemEvent) -> EventLoopCtrl {
        match event {
            // initial setup
            SystemEvent::Init => {
                display::Display::Default().set_refresh_rate(0.);

                // Register our update handler that defined below
                self.set_update_handler();
            }
            _ => {}
        }

        EventLoopCtrl::Continue
    }
}

impl Update for State {
    /// Updates the state
    fn update(&mut self) -> UpdateCtrl {
        const LABEL_DEF: &str = "Just rotating bitmap:\0";
        const ENC: StringEncoding = StringEncoding::ASCII;

        let cstr = CStr::from_bytes_with_nul(LABEL_DEF.as_bytes()).unwrap();

        // Create cached api end-point
        let gfx = Graphics::Cached();

        gfx.clear(Color::WHITE);

        // get width (screen-size) of text
        let font = Default::default();
        let text_width = gfx.get_text_width_cstr(cstr, ENC, font, 0);

        // render text
        gfx.draw_text_cstr(
            cstr,
            ENC,
            CENTER_X as c_int - text_width / 2,
            TEXT_HEIGHT.try_into().unwrap(),
        );

        let rotation = self.crank.angle();
        let buttons = self.buttons.get();

        const SPEED: i32 = 4;
        if buttons.current.right() {
            self.pos.x += SPEED;
        }
        if buttons.current.left() {
            self.pos.x -= SPEED;
        }
        if buttons.current.up() {
            self.pos.y -= SPEED;
        }
        if buttons.current.down() {
            self.pos.y += SPEED;
        }

        // Check screen boundaries
        let bitmap_data = self.image.bitmap_data();
        let width = bitmap_data.width / 2;
        let height = bitmap_data.height / 2;
        if self.pos.x < width {
            self.pos.x = width
        } else if self.pos.x > Display::COLUMNS as i32 - width {
            self.pos.x = Display::COLUMNS as i32 - width
        }
        if self.pos.y < height {
            self.pos.y = height
        } else if self.pos.y > Display::ROWS as i32 - height {
            self.pos.y = Display::ROWS as i32 - height
        }

        // draw bitmap
        self.image
            .draw_rotated(self.pos.x, self.pos.y, rotation, 0.5, 0.5, 1.0, 1.0);

        UpdateCtrl::Continue
    }
}

/// Entry point / event handler
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
fn event_handler(_: NonNull<PlaydateAPI>, event: SystemEvent, _: u32) -> EventLoopCtrl {
    // Unsafe static storage for our state.
    // Usually it's safe because there's only one thread.
    pub static mut STATE: Option<State> = None;
    if unsafe { STATE.is_none() } {
        let state = State::new();
        unsafe { STATE = Some(state) }
    }

    // Call state.event
    unsafe { STATE.as_mut() }.expect("impossible").event(event)
}

// Needed for debug build
ll_symbols!();
