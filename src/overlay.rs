//! Web-content overlay. Displays a web-page within an area of the main window.
use crate::view::View;

/// Web-content overlay. Displays a web-page within an area of the main window.
///
/// Each `Overlay` is essentially a View and an on-screen quad. You should
/// create the Overlay then load content into the underlying View.
///
/// Can be created with [`Window::create_overlay`](crate::window::Window::create_overlay)
/// or [`Window::create_overlay_with_view`](crate::window::Window::create_overlay_with_view).
pub struct Overlay {
    internal: ul_sys::ULOverlay,

    view: View,
}

impl Overlay {
    /// Internal function helper to create an overlay.
    /// (See [`Window::create_overlay`](crate::window::Window::create_overlay))
    pub(crate) unsafe fn create(
        window: ul_sys::ULWindow,
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> Option<Self> {
        let internal_overlay = ul_sys::ulCreateOverlay(window, width, height, x, y);

        if internal_overlay.is_null() {
            return None;
        }

        let raw_view = ul_sys::ulOverlayGetView(internal_overlay);
        // the overlay owns the view, we can't need to destroy it on drop
        let view = View::from_raw(raw_view)?;
        Some(Self {
            internal: internal_overlay,
            view,
        })
    }

    /// Internal function helper to create an overlay with a view
    /// (See [`Window::create_overlay_with_view`](crate::window::Window::create_overlay_with_view))
    pub(crate) unsafe fn create_with_view(
        window_raw: ul_sys::ULWindow,
        view: View,
        x: i32,
        y: i32,
    ) -> Option<Self> {
        let internal = ul_sys::ulCreateOverlayWithView(window_raw, view.to_ul(), x, y);
        if internal.is_null() {
            return None;
        }

        Some(Self { internal, view })
    }
}

impl Overlay {
    /// Get the underlying View.
    pub fn view(&self) -> &View {
        &self.view
    }

    /// Get the width (in pixels).
    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetWidth(self.internal) }
    }

    /// Get the height (in pixels).
    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetHeight(self.internal) }
    }

    /// Get the x-position (offset from the left of the Window), in pixels.
    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetX(self.internal) }
    }

    /// Get the y-position (offset from the top of the Window), in pixels.
    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetY(self.internal) }
    }

    /// Whether or not the overlay is hidden (not drawn).
    pub fn is_hidden(&self) -> bool {
        unsafe { ul_sys::ulOverlayIsHidden(self.internal) }
    }

    /// Show the overlay.
    pub fn show(&self) {
        unsafe { ul_sys::ulOverlayShow(self.internal) }
    }

    /// Hide the overlay (will no longer be drawn)
    pub fn hide(&self) {
        unsafe { ul_sys::ulOverlayHide(self.internal) }
    }

    /// Whether or not this overlay has keyboard focus.
    pub fn has_focus(&self) -> bool {
        unsafe { ul_sys::ulOverlayHasFocus(self.internal) }
    }

    /// Grant this overlay exclusive keyboard focus.
    pub fn focus(&self) {
        unsafe { ul_sys::ulOverlayFocus(self.internal) }
    }

    /// Remove keyboard focus.
    pub fn unfocus(&self) {
        unsafe { ul_sys::ulOverlayUnfocus(self.internal) }
    }

    /// Move the overlay to a new position (in pixels).
    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulOverlayMoveTo(self.internal, x, y) }
    }

    /// Resize the overlay (and underlying View), dimensions should be
    /// specified in pixels.
    pub fn resize(&self, width: u32, height: u32) {
        unsafe { ul_sys::ulOverlayResize(self.internal, width, height) }
    }

    // only found in C++ and not in the C API yet.
    // pub fn need_repaint(&self) -> bool {
    // }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyOverlay(self.internal);
        }
    }
}
