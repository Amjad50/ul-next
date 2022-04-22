use crate::rect::Rect;

pub struct Surface {
    internal: ul_sys::ULSurface,
}

impl Surface {
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULSurface) -> Self {
        Self { internal: raw }
    }
}

impl Surface {
    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulSurfaceGetWidth(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulSurfaceGetHeight(self.internal) }
    }

    pub fn row_bytes(&self) -> u32 {
        unsafe { ul_sys::ulSurfaceGetRowBytes(self.internal) }
    }

    pub fn bytes_size(&self) -> u64 {
        unsafe { ul_sys::ulSurfaceGetSize(self.internal) }
    }

    // TODO: add execulsive lock and error handling
    pub fn lock_pixels(&mut self) -> &mut [u8] {
        let raw_locked_pixels = unsafe { ul_sys::ulSurfaceLockPixels(self.internal) };
        let size = self.bytes_size() as usize;
        unsafe { std::slice::from_raw_parts_mut(raw_locked_pixels as _, size) }
    }

    // TODO: make it part of drop for locked pixels
    pub fn unlock_pixels(&mut self) {
        unsafe { ul_sys::ulSurfaceUnlockPixels(self.internal) }
    }

    // TODO: cannot be called on locked surface
    pub fn resize(&self, width: u32, height: u32) {
        unsafe { ul_sys::ulSurfaceResize(self.internal, width, height) }
    }

    pub fn set_dirty_bounds(&self, bounds: Rect<i32>) {
        unsafe { ul_sys::ulSurfaceSetDirtyBounds(self.internal, bounds.into()) }
    }

    pub fn dirty_bounds(&self) -> Rect<i32> {
        unsafe { ul_sys::ulSurfaceGetDirtyBounds(self.internal).into() }
    }

    pub fn clear_dirty_bounds(&self) {
        unsafe { ul_sys::ulSurfaceClearDirtyBounds(self.internal) }
    }

    //pub fn user_data(&self) -> *mut std::ffi::c_void {
    //    unsafe { ul_sys::ulSurfaceGetUserData(self.internal) }
    //}
}
