//! `Window` to display web content in an [`App`].
//!
//! An [`App`] can have multiple [`Window`]s, and each [`Window`] can
//! have multiple [`Overlay`]s managing multiple [`View`]s.
//!
//! [`App`]: crate::app::App

use std::{self, ffi::CString};

use crate::{overlay::Overlay, view::View};

/// Cursor types (See [`View::set_change_cursor_callback`])
#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    Alias = ul_sys::ULCursor_kCursor_Alias as isize,
    Cell = ul_sys::ULCursor_kCursor_Cell as isize,
    ColumnResize = ul_sys::ULCursor_kCursor_ColumnResize as isize,
    ContextMenu = ul_sys::ULCursor_kCursor_ContextMenu as isize,
    Copy = ul_sys::ULCursor_kCursor_Copy as isize,
    Cross = ul_sys::ULCursor_kCursor_Cross as isize,
    Custom = ul_sys::ULCursor_kCursor_Custom as isize,
    EastPanning = ul_sys::ULCursor_kCursor_EastPanning as isize,
    EastResize = ul_sys::ULCursor_kCursor_EastResize as isize,
    EastWestResize = ul_sys::ULCursor_kCursor_EastWestResize as isize,
    Grab = ul_sys::ULCursor_kCursor_Grab as isize,
    Grabbing = ul_sys::ULCursor_kCursor_Grabbing as isize,
    Hand = ul_sys::ULCursor_kCursor_Hand as isize,
    Help = ul_sys::ULCursor_kCursor_Help as isize,
    IBeam = ul_sys::ULCursor_kCursor_IBeam as isize,
    MiddlePanning = ul_sys::ULCursor_kCursor_MiddlePanning as isize,
    Move = ul_sys::ULCursor_kCursor_Move as isize,
    NoDrop = ul_sys::ULCursor_kCursor_NoDrop as isize,
    None = ul_sys::ULCursor_kCursor_None as isize,
    NorthEastPanning = ul_sys::ULCursor_kCursor_NorthEastPanning as isize,
    NorthEastResize = ul_sys::ULCursor_kCursor_NorthEastResize as isize,
    NorthEastSouthWestResize = ul_sys::ULCursor_kCursor_NorthEastSouthWestResize as isize,
    NorthPanning = ul_sys::ULCursor_kCursor_NorthPanning as isize,
    NorthResize = ul_sys::ULCursor_kCursor_NorthResize as isize,
    NorthSouthResize = ul_sys::ULCursor_kCursor_NorthSouthResize as isize,
    NorthWestPanning = ul_sys::ULCursor_kCursor_NorthWestPanning as isize,
    NorthWestResize = ul_sys::ULCursor_kCursor_NorthWestResize as isize,
    NorthWestSouthEastResize = ul_sys::ULCursor_kCursor_NorthWestSouthEastResize as isize,
    NotAllowed = ul_sys::ULCursor_kCursor_NotAllowed as isize,
    Pointer = ul_sys::ULCursor_kCursor_Pointer as isize,
    Progress = ul_sys::ULCursor_kCursor_Progress as isize,
    RowResize = ul_sys::ULCursor_kCursor_RowResize as isize,
    SouthEastPanning = ul_sys::ULCursor_kCursor_SouthEastPanning as isize,
    SouthEastResize = ul_sys::ULCursor_kCursor_SouthEastResize as isize,
    SouthPanning = ul_sys::ULCursor_kCursor_SouthPanning as isize,
    SouthResize = ul_sys::ULCursor_kCursor_SouthResize as isize,
    SouthWestPanning = ul_sys::ULCursor_kCursor_SouthWestPanning as isize,
    SouthWestResize = ul_sys::ULCursor_kCursor_SouthWestResize as isize,
    VerticalText = ul_sys::ULCursor_kCursor_VerticalText as isize,
    Wait = ul_sys::ULCursor_kCursor_Wait as isize,
    WestPanning = ul_sys::ULCursor_kCursor_WestPanning as isize,
    WestResize = ul_sys::ULCursor_kCursor_WestResize as isize,
    ZoomIn = ul_sys::ULCursor_kCursor_ZoomIn as isize,
    ZoomOut = ul_sys::ULCursor_kCursor_ZoomOut as isize,
}

impl TryFrom<ul_sys::ULCursor> for Cursor {
    type Error = ();

    fn try_from(value: ul_sys::ULCursor) -> Result<Self, Self::Error> {
        match value {
            ul_sys::ULCursor_kCursor_Alias => Ok(Self::Alias),
            ul_sys::ULCursor_kCursor_Cell => Ok(Self::Cell),
            ul_sys::ULCursor_kCursor_ColumnResize => Ok(Self::ColumnResize),
            ul_sys::ULCursor_kCursor_ContextMenu => Ok(Self::ContextMenu),
            ul_sys::ULCursor_kCursor_Copy => Ok(Self::Copy),
            ul_sys::ULCursor_kCursor_Cross => Ok(Self::Cross),
            ul_sys::ULCursor_kCursor_Custom => Ok(Self::Custom),
            ul_sys::ULCursor_kCursor_EastPanning => Ok(Self::EastPanning),
            ul_sys::ULCursor_kCursor_EastResize => Ok(Self::EastResize),
            ul_sys::ULCursor_kCursor_EastWestResize => Ok(Self::EastWestResize),
            ul_sys::ULCursor_kCursor_Grab => Ok(Self::Grab),
            ul_sys::ULCursor_kCursor_Grabbing => Ok(Self::Grabbing),
            ul_sys::ULCursor_kCursor_Hand => Ok(Self::Hand),
            ul_sys::ULCursor_kCursor_Help => Ok(Self::Help),
            ul_sys::ULCursor_kCursor_IBeam => Ok(Self::IBeam),
            ul_sys::ULCursor_kCursor_MiddlePanning => Ok(Self::MiddlePanning),
            ul_sys::ULCursor_kCursor_Move => Ok(Self::Move),
            ul_sys::ULCursor_kCursor_NoDrop => Ok(Self::NoDrop),
            ul_sys::ULCursor_kCursor_None => Ok(Self::None),
            ul_sys::ULCursor_kCursor_NorthEastPanning => Ok(Self::NorthEastPanning),
            ul_sys::ULCursor_kCursor_NorthEastResize => Ok(Self::NorthEastResize),
            ul_sys::ULCursor_kCursor_NorthEastSouthWestResize => Ok(Self::NorthEastSouthWestResize),
            ul_sys::ULCursor_kCursor_NorthPanning => Ok(Self::NorthPanning),
            ul_sys::ULCursor_kCursor_NorthResize => Ok(Self::NorthResize),
            ul_sys::ULCursor_kCursor_NorthSouthResize => Ok(Self::NorthSouthResize),
            ul_sys::ULCursor_kCursor_NorthWestPanning => Ok(Self::NorthWestPanning),
            ul_sys::ULCursor_kCursor_NorthWestResize => Ok(Self::NorthWestResize),
            ul_sys::ULCursor_kCursor_NorthWestSouthEastResize => Ok(Self::NorthWestSouthEastResize),
            ul_sys::ULCursor_kCursor_NotAllowed => Ok(Self::NotAllowed),
            ul_sys::ULCursor_kCursor_Pointer => Ok(Self::Pointer),
            ul_sys::ULCursor_kCursor_Progress => Ok(Self::Progress),
            ul_sys::ULCursor_kCursor_RowResize => Ok(Self::RowResize),
            ul_sys::ULCursor_kCursor_SouthEastPanning => Ok(Self::SouthEastPanning),
            ul_sys::ULCursor_kCursor_SouthEastResize => Ok(Self::SouthEastResize),
            ul_sys::ULCursor_kCursor_SouthPanning => Ok(Self::SouthPanning),
            ul_sys::ULCursor_kCursor_SouthResize => Ok(Self::SouthResize),
            ul_sys::ULCursor_kCursor_SouthWestPanning => Ok(Self::SouthWestPanning),
            ul_sys::ULCursor_kCursor_SouthWestResize => Ok(Self::SouthWestResize),
            ul_sys::ULCursor_kCursor_VerticalText => Ok(Self::VerticalText),
            ul_sys::ULCursor_kCursor_Wait => Ok(Self::Wait),
            ul_sys::ULCursor_kCursor_WestPanning => Ok(Self::WestPanning),
            ul_sys::ULCursor_kCursor_WestResize => Ok(Self::WestResize),
            ul_sys::ULCursor_kCursor_ZoomIn => Ok(Self::ZoomIn),
            ul_sys::ULCursor_kCursor_ZoomOut => Ok(Self::ZoomOut),
            _ => Err(()),
        }
    }
}

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
    internal: ul_sys::ULWindow,
    need_to_destroy: bool,
}

impl Window {
    /// Internal function helper to create a view.
    /// (See [`App::create_window`](crate::app::App::create_window))
    pub(crate) unsafe fn create(
        monitor_raw: ul_sys::ULMonitor,
        width: u32,
        height: u32,
        fullscreen: bool,
        window_flags: WindowFlags,
    ) -> Self {
        let internal = ul_sys::ulCreateWindow(
            monitor_raw,
            width,
            height,
            fullscreen,
            window_flags.to_u32(),
        );

        Self {
            internal,
            need_to_destroy: true,
        }
    }

    /// Helper internal function to allow getting a reference to a managed
    /// session.
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULWindow) -> Self {
        Self {
            internal: raw,
            need_to_destroy: false,
        }
    }

    /// Get the window width (in screen coordinates).
    pub fn screen_width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenWidth(self.internal) }
    }

    /// Get the window width (in pixels).
    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetWidth(self.internal) }
    }

    /// Get the window height (in screen coordinates).
    pub fn screen_height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenHeight(self.internal) }
    }

    /// Get the window height (in pixels).
    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetHeight(self.internal) }
    }

    /// Move the window to a new position (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulWindowMoveTo(self.internal, x, y) }
    }

    /// Move the window to the center of the monitor.
    pub fn move_to_center(&self) {
        unsafe { ul_sys::ulWindowMoveToCenter(self.internal) }
    }

    /// Get the x-position of the window (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionX(self.internal) }
    }

    /// Get the y-position of the window (in screen coordinates) relative
    /// to the top-left of the monitor area.
    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionY(self.internal) }
    }

    /// Whether or not the window is fullscreen.
    pub fn is_fullscreen(&self) -> bool {
        unsafe { ul_sys::ulWindowIsFullscreen(self.internal) }
    }

    /// The DPI scale of the window.
    pub fn scale(&self) -> f64 {
        unsafe { ul_sys::ulWindowGetScale(self.internal) }
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        let c_string = CString::new(title).unwrap();
        unsafe { ul_sys::ulWindowSetTitle(self.internal, c_string.as_ptr()) }
    }

    /// Set the cursor.
    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe { ul_sys::ulWindowSetCursor(self.internal, cursor as u32) }
    }

    /// Show the window (if it was previously hidden).
    pub fn show(&self) {
        unsafe { ul_sys::ulWindowShow(self.internal) }
    }

    /// Hide the window.
    pub fn hide(&self) {
        unsafe { ul_sys::ulWindowHide(self.internal) }
    }

    /// Whether or not the window is currently visible (not hidden).
    pub fn is_visible(&self) -> bool {
        unsafe { ul_sys::ulWindowIsVisible(self.internal) }
    }

    /// Close the window.
    pub fn close(&self) {
        unsafe { ul_sys::ulWindowClose(self.internal) }
    }

    /// Convert screen coordinates to pixels using the current DPI scale.
    pub fn screen_to_pixels(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowScreenToPixels(self.internal, val) }
    }

    /// Convert pixels to screen coordinates using the current DPI scale.
    pub fn pixels_to_screen(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowPixelsToScreen(self.internal, val) }
    }

    set_callback! {
        /// Called when the Window is closed.
        ///
        /// # Callback Arguments
        /// * `window: &Window` - The window that fired the event (eg. self)
        pub fn set_close_callback(&self, callback: FnMut(window: &Window)) :
           ulWindowSetCloseCallback(ul_window: ul_sys::ULWindow) {
               let window = &Window::from_raw(ul_window);
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
            ulWindowSetResizeCallback(ul_window: ul_sys::ULWindow, width: u32, height: u32) {
               let window = &Window::from_raw(ul_window);
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
    pub fn create_overlay(&self, width: u32, height: u32, x: i32, y: i32) -> Overlay {
        unsafe { Overlay::create(self.internal, width, height, x, y) }
    }

    /// Create a new Overlay, wrapping an existing view.
    ///
    /// # Arguments
    /// * `view` - The view to wrap (will use its width and height).
    /// * `x` - The x-position (offset from the left of this Window), in pixels.
    /// * `y` - The y-position (offset from the top of this Window), in pixels.
    pub fn create_overlay_with_view(&self, view: View, x: i32, y: i32) -> Overlay {
        unsafe { Overlay::create_with_view(self.internal, view, x, y) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe { ul_sys::ulDestroyWindow(self.internal) }
        }
    }
}
