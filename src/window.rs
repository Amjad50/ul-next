use std::{self, ffi::CString};

use crate::{overlay::Overlay, view::View};

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

pub struct WindowFlags {
    pub borderless: bool,
    pub titled: bool,
    pub resizable: bool,
    pub maximizable: bool,
    pub hidden: bool,
}

impl WindowFlags {
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

pub struct Window {
    internal: ul_sys::ULWindow,
    need_to_destroy: bool,
}

impl Window {
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

    pub(crate) unsafe fn from_raw(raw: ul_sys::ULWindow) -> Self {
        Self {
            internal: raw,
            need_to_destroy: false,
        }
    }

    pub fn screen_width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenWidth(self.internal) }
    }

    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetWidth(self.internal) }
    }

    pub fn screen_height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenHeight(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetHeight(self.internal) }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulWindowMoveTo(self.internal, x, y) }
    }

    pub fn move_to_center(&self) {
        unsafe { ul_sys::ulWindowMoveToCenter(self.internal) }
    }

    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionX(self.internal) }
    }

    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionY(self.internal) }
    }

    pub fn is_fullscreen(&self) -> bool {
        unsafe { ul_sys::ulWindowIsFullscreen(self.internal) }
    }

    pub fn scale(&self) -> f64 {
        unsafe { ul_sys::ulWindowGetScale(self.internal) }
    }

    pub fn set_title(&self, title: &str) {
        let c_string = CString::new(title).unwrap();
        unsafe { ul_sys::ulWindowSetTitle(self.internal, c_string.as_ptr()) }
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe { ul_sys::ulWindowSetCursor(self.internal, cursor as u32) }
    }

    pub fn show(&self) {
        unsafe { ul_sys::ulWindowShow(self.internal) }
    }

    pub fn hide(&self) {
        unsafe { ul_sys::ulWindowHide(self.internal) }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { ul_sys::ulWindowIsVisible(self.internal) }
    }

    pub fn close(&self) {
        unsafe { ul_sys::ulWindowClose(self.internal) }
    }

    pub fn screen_to_pixels(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowScreenToPixels(self.internal, val) }
    }

    pub fn pixels_to_screen(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowPixelsToScreen(self.internal, val) }
    }

    set_callback! {
        pub fn set_close_callback(&self, callback: FnMut(window: &Window)) :
           ulWindowSetCloseCallback(ul_window: ul_sys::ULWindow) {
               let window = &Window::from_raw(ul_window);
        }
    }

    set_callback! {
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
    pub fn create_overlay(&self, width: u32, height: u32, x: i32, y: i32) -> Overlay {
        unsafe { Overlay::create(self.internal, width, height, x, y) }
    }

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
