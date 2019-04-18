mod ffi {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex};

use failure::Error;

#[derive(Debug)]
pub struct Key {
    name: String,
    code: u32,
    sym: u64,
}

pub struct State<T> {
    data: VecDeque<T>,
    max_len: NonZeroUsize,
}

impl<T> State<T> {
    pub fn new(max_len: NonZeroUsize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_len.get()),
            max_len,
        }
    }
    pub(crate) fn data(&self) -> &VecDeque<T> {
        &self.data
    }
    fn push(&mut self, value: T) {
        if self.data.len() >= self.max_len.get() {
            self.data.pop_back();
        }
        self.data.push_front(value);
    }
}

pub fn log_keys(display: bool, state: Arc<Mutex<State<Key>>>) -> Result<(), Error> {
    let mut buffer = [0 as c_char; 17];
    let mut compose_status = ffi::_XComposeStatus {
        compose_ptr: std::ptr::null_mut() as *mut c_char,
        chars_matched: 0,
    };
    let mut event = ffi::XEvent { type_: 0 };
    let mut key_sym: ffi::KeySym = 0;
    let mut revert: c_int = 0;
    let mut window: ffi::Window = 0;

    unsafe { ffi::XSetErrorHandler(Some(error_handler)) };

    let display_p = unsafe { ffi::XOpenDisplay(std::ptr::null()) };
    if display_p.is_null() {
        failure::bail!("Unable to get handle to display.");
    }

    unsafe { ffi::XGetInputFocus(display_p, &mut window as *mut _, &mut revert as *mut _) };

    unsafe {
        ffi::XSelectInput(
            display_p,
            window,
            (ffi::KeyPressMask | ffi::FocusChangeMask) as _,
        );
    }

    loop {
        unsafe { ffi::XNextEvent(display_p, &mut event as *mut _) };

        if unsafe { event.type_ } == ffi::KeyPress as c_int {
            let mut key_event = unsafe { event.xkey };

            let len = unsafe {
                ffi::XLookupString(
                    &mut key_event as *mut _,
                    buffer[..].as_mut_ptr(),
                    16,
                    &mut key_sym as *mut _,
                    &mut compose_status as *mut _,
                )
            };

            let buffer: [u8; 17] = unsafe { std::mem::transmute(buffer) };

            let key = if len > 0 && !(buffer[0] as char).is_control() {
                Key {
                    code: key_event.keycode,
                    name: keysym_name(key_sym as u32)?,
                    sym: key_sym,
                }
            } else {
                Key {
                    code: key_event.keycode,
                    name: keysym_name(key_sym as u32)?,
                    sym: key_sym,
                }
            };
            if display {
                println!("{:x?}", &key);
            }
            let mut guard = state.lock().map_err(|_| failure::err_msg("Lock failed."))?;
            guard.push(key);
        } else if unsafe { event.type_ } == ffi::FocusOut as c_int {
            unsafe { ffi::XSelectInput(display_p, window, 0) };
            unsafe { ffi::XGetInputFocus(display_p, &mut window as *mut _, &mut revert as *mut _) };
            unsafe {
                ffi::XSelectInput(
                    display_p,
                    window,
                    (ffi::KeyPressMask | ffi::FocusChangeMask) as _,
                )
            };
        }
    }
}

fn keysym_name(keysym: u32) -> Result<String, Error> {
    let mut buf: [c_char; 64] = unsafe { std::mem::uninitialized() };
    let len = unsafe { ffi::xkb_keysym_get_name(keysym, buf.as_mut_ptr(), 64) };
    if len < 0 {
        failure::bail!("{} is not a valid keysym.", keysym)
    }
    let buf: [u8; 64] = unsafe { std::mem::transmute(buf) };
    let s = std::str::from_utf8(&buf[..len as usize])?.to_string();
    Ok(s)
}

extern "C" fn error_handler(_: *mut ffi::Display, e: *mut ffi::XErrorEvent) -> c_int {
    let e = unsafe { &*e };

    if e.error_code == 3 {
        // Ignore BadWindow error
        0
    } else {
        // Exit on any other error
        eprintln!("X11 error: {:?}", e);
        std::process::exit(1);
    }
}
