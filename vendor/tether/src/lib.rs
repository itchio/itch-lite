#![deny(missing_docs)]

//! Windows that are web views.

pub mod raw;

use log::error;
use std::cell::{Cell, RefCell};
use std::ffi::{c_void, CStr, CString};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{panic, process};

thread_local! {
    static MAIN_THREAD: Cell<bool> = Cell::new(false);
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// An event handler; you probably want to implement one.
///
/// - When the webpage calls `window.tether`, the message is passed to `handle`.
/// - The handler is dropped when the window is closed.
pub trait Handler: 'static {
    /// The webpage called `window.tether` with the given string.
    fn handle_rpc(&mut self, window: Window, message: &str) {
        let _ = (window, message);
    }

    /// A request was made, and it can be intercepted
    fn handle_net(&mut self, req: NetRequest) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// A network request made by the webview - could be a page load, an
/// XMLHTTPRequest, a fetch(), an img src, anything
pub struct NetRequest<'a> {
    /// The URI that was requested by the webview
    request_uri: &'a str,
    /// The underlying raw request
    req: &'a raw::tether_net_request,
}

impl<'a> NetRequest<'a> {
    /// Returns the URI that was requested by the webview
    pub fn uri(&self) -> &str {
        self.request_uri
    }

    /// Set the response for this request. bypassing the
    /// regular network stack.
    pub fn respond(self, res: NetResponse) {
        let res = res.into_raw();
        unsafe { (self.req.respond)(self.req.respond_ctx, &res) };
    }

    /// Create a NetRequest instance from its raw counterpart.
    unsafe fn from_raw(
        req: &'a raw::tether_net_request,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            request_uri: CStr::from_ptr(req.request_uri).to_str()?,
            req,
        })
    }
}

/// A network response
pub struct NetResponse<'a> {
    /// Contents of the response
    pub content: &'a [u8],

    /// The HTTP status code
    pub status_code: usize,
}

impl<'a> NetResponse<'a> {
    // TODO: figure out a way to tie those lifetimes
    fn into_raw(&self) -> raw::tether_net_response {
        raw::tether_net_response {
            content: self.content.as_ptr(),
            content_length: self.content.len(),
            status_code: self.status_code,
        }
    }
}

#[derive(Clone)]
/// A window, which may or may not be open.
pub struct Window {
    data: Rc<RefCell<Option<raw::tether>>>,
}

struct Data {
    win: Window,
    handler: Option<Box<dyn Handler>>,
}

impl Window {
    /// Make a new window with the given options.
    pub fn new(opts: Options) -> Self {
        assert_main();

        let this = Window {
            data: Rc::new(RefCell::new(None)),
        };

        let handler = opts.handler;

        let opts = raw::tether_options {
            initial_width: opts.initial_width,
            initial_height: opts.initial_height,
            minimum_width: opts.minimum_width,
            minimum_height: opts.minimum_height,

            borderless: opts.borderless,
            debug: opts.debug,

            data: Box::<Data>::into_raw(Box::new(Data {
                win: this.clone(),
                handler,
            })) as _,
            closed: closed,
            message: message,
            net_request: net_request,
        };

        let raw = unsafe { raw::tether_new(opts) };
        this.data.replace(Some(raw));

        unsafe extern "C" fn net_request(data: *mut c_void, c_req: *const raw::tether_net_request) {
            let process = || -> Result<(), Box<dyn std::error::Error>> {
                let data = data as *mut Data;

                if let Some(handler) = (*data).handler.as_mut() {
                    let req = NetRequest::from_raw(&*c_req)?;
                    handler.handle_net(req)?;
                }

                Ok(())
            };

            abort_on_panic(|| match process() {
                Err(e) => error!("{}", e),
                _ => {} // good!
            });
        }

        unsafe extern "C" fn message(data: *mut c_void, message: *const i8) {
            abort_on_panic(|| {
                let data = data as *mut Data;

                if let Some(handler) = (*data).handler.as_mut() {
                    match CStr::from_ptr(message).to_str() {
                        Ok(message) => {
                            handler.handle_rpc((*data).win.clone(), message);
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
            });
        }

        unsafe extern "C" fn closed(data: *mut c_void) {
            abort_on_panic(|| {
                let _ = Box::<Data>::from_raw(data as _);
            });
        }

        this
    }

    /// Make a new window with the default options and the given handler.
    pub fn with_handler(handler: impl Handler) -> Self {
        Self::new(Options {
            handler: Some(Box::new(handler)),
            ..Default::default()
        })
    }

    /// Evaluate the given JavaScript asynchronously.
    pub fn eval<I: Into<String>>(&self, s: I) {
        if let Some(data) = *self.data.borrow_mut() {
            let s = string_to_cstring(s);
            unsafe {
                raw::tether_eval(data, s.as_ptr());
            }
        }
    }

    /// Load the given HTML asynchronously.
    pub fn load<I: Into<String>>(&self, s: I) {
        if let Some(data) = *self.data.borrow_mut() {
            let s = string_to_cstring(s);
            unsafe {
                raw::tether_load(data, s.as_ptr());
            }
        }
    }

    /// Set this window's title to the given string.
    pub fn title<I: Into<String>>(&self, s: I) {
        if let Some(data) = *self.data.borrow_mut() {
            let s = string_to_cstring(s);
            unsafe {
                raw::tether_title(data, s.as_ptr());
            }
        }
    }

    /// Focus this window above the other windows.
    pub fn focus(&self) {
        if let Some(data) = *self.data.borrow_mut() {
            unsafe {
                raw::tether_focus(data);
            }
        }
    }

    /// Close this window.
    pub fn close(&self) {
        if let Some(data) = *self.data.borrow_mut() {
            unsafe {
                raw::tether_close(data);
            }
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

/// The window options.
///
/// Note that these are mostly *suggestions* rather than *requirements*.
pub struct Options {
    /// The initial window width in pixels.
    pub initial_width: usize,
    /// The initial window height in pixels.
    pub initial_height: usize,
    /// The minimum window width in pixels.
    pub minimum_width: usize,
    /// The minimum window height in pixels.
    pub minimum_height: usize,

    /// Whether to draw the title bar and stuff like that.
    pub borderless: bool,
    /// I'm not entirely sure what enabling this does.
    pub debug: bool,

    /// The window's handler.
    pub handler: Option<Box<dyn Handler>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            initial_width: 640,
            initial_height: 480,
            minimum_width: 480,
            minimum_height: 360,

            borderless: false,
            debug: false,

            handler: None,
        }
    }
}

/// Initialize things; call this first.
///
/// By calling this function, you're promising us that you haven't called it
/// before, and that this is the main thread. The provided callback should
/// contain your "real" main function.
pub unsafe fn start(cb: fn()) {
    static mut INIT: Option<fn()> = None;
    INIT = Some(cb);

    unsafe extern "C" fn init() {
        abort_on_panic(|| {
            MAIN_THREAD.with(|initialized| {
                initialized.set(true);
            });

            INITIALIZED.store(true, Ordering::Relaxed);

            INIT.unwrap()();
        });
    }

    raw::tether_start(Some(init));
}

/// Terminate the application as gracefully as possible.
pub fn exit() {
    assert_main();

    unsafe {
        raw::tether_exit();
    }
}

/// Run the given function on the main thread.
pub fn dispatch<F: FnOnce() + Send>(f: F) {
    assert_initialized();

    unsafe {
        raw::tether_dispatch(Box::<F>::into_raw(Box::new(f)) as _, Some(execute::<F>));
    }

    unsafe extern "C" fn execute<F: FnOnce() + Send>(data: *mut c_void) {
        abort_on_panic(|| {
            Box::<F>::from_raw(data as _)();
        });
    }
}

fn abort_on_panic<F: FnOnce() + panic::UnwindSafe>(f: F) {
    if panic::catch_unwind(f).is_err() {
        process::abort();
    }
}

/// Make sure that we're initialized.
fn assert_initialized() {
    assert!(INITIALIZED.load(Ordering::Relaxed));
}

/// Make sure that we're initialized and on the main thread.
fn assert_main() {
    MAIN_THREAD.with(|initialized| {
        assert!(initialized.get());
    });
}

fn string_to_cstring<I: Into<String>>(s: I) -> CString {
    CString::new(s.into()).unwrap()
}
