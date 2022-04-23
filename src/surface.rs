use std::ops::{Deref, DerefMut};

use crate::rect::Rect;

pub struct PixelsGuard<'a> {
    lock: &'a mut Surface,
    pixels: &'a mut [u8],
}

impl<'a> PixelsGuard<'a> {
    unsafe fn new(lock: &'a mut Surface, pixels: &'a mut [u8]) -> PixelsGuard<'a> {
        PixelsGuard { lock, pixels }
    }
}

impl Deref for PixelsGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.pixels
    }
}

impl DerefMut for PixelsGuard<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
}

impl Drop for PixelsGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.lock.raw_unlock_pixels();
        }
    }
}

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

    // TODO: add error handling
    pub fn lock_pixels<'a>(&'a mut self) -> PixelsGuard<'a> {
        let raw_locked_pixels = unsafe { ul_sys::ulSurfaceLockPixels(self.internal) };
        let size = self.bytes_size() as usize;
        unsafe {
            let data = std::slice::from_raw_parts_mut(raw_locked_pixels as _, size);
            PixelsGuard::new(self, data)
        }
    }

    // TODO: make it part of drop for locked pixels
    pub(crate) unsafe fn raw_unlock_pixels(&self) {
        ul_sys::ulSurfaceUnlockPixels(self.internal)
    }

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
