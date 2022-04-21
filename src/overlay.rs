use crate::view::View;

pub struct Overlay {
    internal: ul_sys::ULOverlay,

    view: View,
}

impl Overlay {
    pub(crate) unsafe fn create(
        window: ul_sys::ULWindow,
        height: u32,
        width: u32,
        x: i32,
        y: i32,
    ) -> Self {
        let overlay = ul_sys::ulCreateOverlay(window, width, height, x, y);
        // the overlay owns the view, we can't need to destroy it on drop
        let view = View::from_raw(ul_sys::ulOverlayGetView(overlay), false);
        Self {
            internal: overlay,
            view,
        }
    }

    pub(crate) unsafe fn create_with_view(
        window_raw: ul_sys::ULWindow,
        view: View,
        x: i32,
        y: i32,
    ) -> Self {
        let overlay = ul_sys::ulCreateOverlayWithView(window_raw, view.to_ul(), x, y);
        Self {
            internal: overlay,
            view,
        }
    }
}

impl Overlay {
    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetWidth(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetHeight(self.internal) }
    }

    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetX(self.internal) }
    }

    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetY(self.internal) }
    }

    pub fn is_hidden(&self) -> bool {
        unsafe { ul_sys::ulOverlayIsHidden(self.internal) }
    }

    pub fn show(&self) {
        unsafe { ul_sys::ulOverlayShow(self.internal) }
    }

    pub fn hide(&self) {
        unsafe { ul_sys::ulOverlayHide(self.internal) }
    }

    pub fn has_focus(&self) -> bool {
        unsafe { ul_sys::ulOverlayHasFocus(self.internal) }
    }

    pub fn focus(&self) {
        unsafe { ul_sys::ulOverlayFocus(self.internal) }
    }

    pub fn unfocus(&self) {
        unsafe { ul_sys::ulOverlayUnfocus(self.internal) }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulOverlayMoveTo(self.internal, x, y) }
    }

    pub fn resize(&self, width: u32, height: u32) {
        unsafe { ul_sys::ulOverlayResize(self.internal, width, height) }
    }

    //pub fn need_repaint(&self) -> bool {
    //}
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyOverlay(self.internal);
        }
    }
}
