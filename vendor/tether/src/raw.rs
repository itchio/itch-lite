#![allow(nonstandard_style)]

use std::os::raw::{c_char, c_void};

// !!!
// This should be kept in sync with "native/tether.h"
//
// Doing so manually avoids the dependency on "bindgen"
// !!!

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _tether {
    _unused: [u8; 0],
}

pub type tether = *mut _tether;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tether_options {
    pub initial_width: usize,
    pub initial_height: usize,
    pub minimum_width: usize,
    pub minimum_height: usize,
    pub borderless: bool,
    pub debug: bool,
    pub data: *mut c_void,
    pub message: Option<unsafe extern "C" fn(data: *mut c_void, message: *const c_char)>,
    pub closed: Option<unsafe extern "C" fn(data: *mut c_void)>,
}

pub type tether_start_callback = unsafe extern "C" fn();
pub type tether_dispatch_callback = unsafe extern "C" fn(data: *mut c_void);

extern "C" {
    pub fn tether_start(func: Option<tether_start_callback>);
    pub fn tether_dispatch(data: *mut c_void, func: Option<tether_dispatch_callback>);
    pub fn tether_exit();
    pub fn tether_new(opts: tether_options) -> tether;
    pub fn tether_eval(self_: tether, js: *const c_char);
    pub fn tether_load(self_: tether, html: *const c_char);
    pub fn tether_title(self_: tether, title: *const c_char);
    pub fn tether_focus(self_: tether);
    pub fn tether_close(self_: tether);
}
