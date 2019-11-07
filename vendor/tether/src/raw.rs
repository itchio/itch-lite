#![deny(missing_docs)]
#![allow(nonstandard_style)]

//! C interface required for tether to work.
//!
//! This is implemented in C, C++ and Objective-C for various
//! platforms, see the `native/` source folder.

use std::os::raw::{c_char, c_void};

/// A reference to a window.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _tether {
    _unused: [u8; 0],
}

/// Pointer type for tether windows
pub type tether = *mut _tether;

/// Configuration options for a window.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tether_options {
    /// Initial width of the window in pixels (TODO: figure out HIDPI).
    pub initial_width: usize,
    /// Initial height of the window in pixels (TODO: figure out HIDPI).
    pub initial_height: usize,
    /// Width below which the window cannot be resized.
    pub minimum_width: usize,
    /// Height below which the window cannot be resized.
    pub minimum_height: usize,
    /// When set, don't show OS decorations.
    pub borderless: bool,
    /// When set, enable debug interface, for example Microsoft Edge DevTools Preview
    pub debug: bool,
    /// The data to pass to event handlers.
    pub data: *mut c_void,
    /// The window received a message via `window.tether(string)`.
    pub message: Option<unsafe extern "C" fn(data: *mut c_void, message: *const c_char)>,
    /// The window was closed, and its resources have all been released.
    pub closed: Option<unsafe extern "C" fn(data: *mut c_void)>,
}

extern "C" {
    /// Start the main loop and call the given function.
    ///
    /// This function should be called on the main thread, and at most once. It
    /// should be called before any other `tether` function is called.
    pub fn tether_start(func: Option<unsafe extern "C" fn()>);

    /// Schedule a function to be called on the main thread.
    ///
    /// All the `tether` functions should only be called on the main thread.
    pub fn tether_dispatch(
        data: *mut c_void,
        func: Option<unsafe extern "C" fn(data: *mut c_void)>,
    );

    /// Stop the main loop as gracefully as possible.
    pub fn tether_exit();

    /// Open a new window with the given options.
    pub fn tether_new(opts: tether_options) -> tether;

    /// Run the given script.
    pub fn tether_eval(self_: tether, js: *const c_char);

    /// Display the given HTML.
    pub fn tether_load(self_: tether, html: *const c_char);

    /// Set the window's title.
    pub fn tether_title(self_: tether, title: *const c_char);

    /// Focus the window, and move it in front of the other windows.
    ///
    /// This function will not steal the focus from other applications.
    pub fn tether_focus(self_: tether);

    /// Close the window.
    pub fn tether_close(self_: tether);
}
