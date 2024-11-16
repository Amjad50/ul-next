//! `Window` to display web content in an [`App`].
//!
//! An [`App`] can have multiple [`Window`]s, and each [`Window`] can
//! have multiple [`Overlay`]s managing multiple [`View`]s.
//!
//! [`App`]: crate::app::App

use std::{self, ffi::CString, sync::Arc};

use crate::{overlay::Overlay, view::Cursor, view::View, Library};

/// Window creation flags
pub struct WindowFlags {
    /// Whether the window has borders or not
    pub borderless: bool,
    /// Whether the window has title or not
    pub titled: bool,
    /// Whether the window is resizable or not
    pub resizable: bool,
    /// Whether the window is maximizable or not
    pub maximizable: bool,
    /// The initial hide/show state of the window
    pub hidden: bool,
}

impl WindowFlags {
    /// Helper to convert this struct to a union enum for the C API
    fn to_u32(&self) -> u32 {
        let mut n = 0;

        if self.borderless {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Borderless;
        }
        if self.titled {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Titled;
        }
        if self.resizable {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Resizable;
        }
        if self.maximizable {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Maximizable;
        }
        if self.hidden {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Hidden;
        }

        n
    }
}

/// Window struct, represents a platform window.
pub struct Window {
    lib: Arc<Library>,
    internal: ul_sys::ULWindow,
    need_to_destroy: bool,
}

impl Window {
    /// Internal function helper to create a view.
    /// (See [`App::create_window`](crate::app::App::create_window))
    ///
    /// Returns [`None`] if failed to create the window.
    pub(crate) unsafe fn create(
        lib: Arc<Library>,
        monitor_raw: ul_sys::ULMonitor,
        width: u32,
        height: u32,
        fullscreen: bool,
        window_flags: WindowFlags,
    ) -> Option<Self> {
        let internal = lib.appcore().ulCreateWindow(
            monitor_raw,
            width,
            height,
            fullscreen,
            window_flags.to_u32(),
        );

        if internal.is_null() {
            None
        } else {
            Some(Self {
                lib,
                internal,
                need_to_destroy: true,
            })
        }
    }

    /// Helper internal function to allow getting a reference to a managed
    /// session.
    pub(crate) unsafe fn from_raw(lib: Arc<Library>, raw: ul_sys::ULWindow) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self {
                lib,
                internal: raw,
                need_to_destroy: false,
            })
        }
    }

    /// Get the window width (in screen coordinates).
    pub fn screen_width(&self) -> u32 {
        unsafe { self.lib.appcore().ulWindowGetScreenWidth(self.internal) }
    }

    /// Get the window width (in pixels).
    pub fn width(&self) -> u32 {
        unsafe { self.lib.appcore().ulWindowGetWidth(self.internal) }
    }

    /// Get the window height (in screen coordinates).
    pub fn screen_height(&self) -> u32 {
        unsafe { self.lib.appcore().ulWindowGetScreenHeight(self.internal) }
    }

    /// Get the window height (in pixels).
    pub fn height(&self) -> u32 {
        unsafe { self.lib.appcore().ulWindowGetHeight(self.internal) }
    }

    /// Move the window to a new position (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { self.lib.appcore().ulWindowMoveTo(self.internal, x, y) }
    }

    /// Move the window to the center of the monitor.
    pub fn move_to_center(&self) {
        unsafe { self.lib.appcore().ulWindowMoveToCenter(self.internal) }
    }

    /// Get the x-position of the window (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn x(&self) -> i32 {
        unsafe { self.lib.appcore().ulWindowGetPositionX(self.internal) }
    }

    /// Get the y-position of the window (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn y(&self) -> i32 {
        unsafe { self.lib.appcore().ulWindowGetPositionY(self.internal) }
    }

    /// Whether or not the window is fullscreen.
    pub fn is_fullscreen(&self) -> bool {
        unsafe { self.lib.appcore().ulWindowIsFullscreen(self.internal) }
    }

    /// The DPI scale of the window.
    pub fn scale(&self) -> f64 {
        unsafe { self.lib.appcore().ulWindowGetScale(self.internal) }
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        let c_string = CString::new(title).unwrap();
        unsafe {
            self.lib
                .appcore()
                .ulWindowSetTitle(self.internal, c_string.as_ptr())
        }
    }

    /// Set the cursor.
    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe {
            self.lib
                .appcore()
                .ulWindowSetCursor(self.internal, cursor as u32)
        }
    }

    /// Show the window (if it was previously hidden).
    pub fn show(&self) {
        unsafe { self.lib.appcore().ulWindowShow(self.internal) }
    }

    /// Hide the window.
    pub fn hide(&self) {
        unsafe { self.lib.appcore().ulWindowHide(self.internal) }
    }

    /// Whether or not the window is currently visible (not hidden).
    pub fn is_visible(&self) -> bool {
        unsafe { self.lib.appcore().ulWindowIsVisible(self.internal) }
    }

    /// Close the window.
    pub fn close(&self) {
        unsafe { self.lib.appcore().ulWindowClose(self.internal) }
    }

    /// Convert screen coordinates to pixels using the current DPI scale.
    pub fn screen_to_pixels(&self, val: i32) -> i32 {
        unsafe {
            self.lib
                .appcore()
                .ulWindowScreenToPixels(self.internal, val)
        }
    }

    /// Convert pixels to screen coordinates using the current DPI scale.
    pub fn pixels_to_screen(&self, val: i32) -> i32 {
        unsafe {
            self.lib
                .appcore()
                .ulWindowPixelsToScreen(self.internal, val)
        }
    }

    set_callback! {
        /// Called when the Window is closed.
        ///
        /// # Callback Arguments
        /// * `window: &Window` - The window that fired the event (eg. self)
        pub fn set_close_callback(&self, callback: FnMut(window: &Window)) :
            [Window::lib.appcore()][s] ulWindowSetCloseCallback(ul_window: ul_sys::ULWindow) {
               let window = &Window::from_raw(s.lib.clone(), ul_window).unwrap();
        }
    }

    set_callback! {
        /// Called when the Window is resized.
        ///
        /// # Callback Arguments
        /// * `window: &Window` - The window that fired the event (eg. self)
        /// * `width: u32` - The new width of the window (in pixels)
        /// * `height: u32` - The new height of the window (in pixels)
        pub fn set_resize_callback(&self, callback: FnMut(window: &Window, width: u32, height: u32)) :
            [Window::lib.appcore()][s] ulWindowSetResizeCallback(ul_window: ul_sys::ULWindow, width: u32, height: u32) {
               let window = &Window::from_raw(s.lib.clone(), ul_window).unwrap();
        }
    }

    //pub fn is_accelerated(&self) -> bool {
    //}
    //
    //pub fn render_buff_id(&self) -> u32 {
    //}
    //
    //pub fn draw_surface(&self, surface: &Surface) {
    //}
}

impl Window {
    /// Create a new Overlay.
    ///
    /// # Arguments
    /// * `width` - The width in pixels.
    /// * `height` - The height in pixels.
    /// * `x` - The x-position (offset from the left of this Window), in pixels.
    /// * `y` - The y-position (offset from the top of this Window), in pixels.
    ///
    /// Returns [`None`] if failed to create [`Overlay`].
    pub fn create_overlay(&self, width: u32, height: u32, x: i32, y: i32) -> Option<Overlay> {
        unsafe { Overlay::create(self.lib.clone(), self.internal, width, height, x, y) }
    }

    /// Create a new Overlay, wrapping an existing view.
    ///
    /// # Arguments
    /// * `view` - The view to wrap (will use its width and height).
    /// * `x` - The x-position (offset from the left of this Window), in pixels.
    /// * `y` - The y-position (offset from the top of this Window), in pixels.
    ///
    /// Returns [`None`] if failed to create [`Overlay`].
    pub fn create_overlay_with_view(&self, view: View, x: i32, y: i32) -> Option<Overlay> {
        unsafe { Overlay::create_with_view(self.lib.clone(), self.internal, view, x, y) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe { self.lib.appcore().ulDestroyWindow(self.internal) }
        }
    }
}
