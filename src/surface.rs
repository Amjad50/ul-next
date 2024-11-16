//! Offscreen pixel buffer surface.
//!
//! `Surface`s are used only when the [`View`](crate::view::View) is not accelerated.

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{rect::Rect, Library};

/// An RAII implementation of a “scoped lock” of a pixel buffer for [`Surface`].
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
///
/// This struct is created by [`Surface::lock_pixels`].
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
        self.pixels
    }
}

impl DerefMut for PixelsGuard<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.pixels
    }
}

impl Drop for PixelsGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.lock.raw_unlock_pixels();
        }
    }
}

/// Offscreen pixel buffer surface. (Premultiplied BGRA 32-bit format)
///
/// When using the CPU renderer, each View is painted to its own Surface.
///
// TODO: add support and remove comments below (note //)
/// **NOTE: Custom Surface is currently not support in this Rust wrapper**
//
// You can provide your own Surface implementation to make the renderer paint directly to a block
// of memory controlled by you (this is useful for lower-latency uploads to GPU memory or other
// platform-specific bitmaps).
//
// A default Surface implementation, `BitmapSurface`, is automatically provided by the library when
// you call [`Renderer::create`](crate::renderer::Renderer::create)
// without defining a custom `SurfaceFactory`.
//
// To provide your own custom Surface implementation, you should implement the
// `SurfaceFactory` trait, and pass the struct to `platform::set_surface_definition`
// before calling [`Renderer::create`](crate::renderer::Renderer::create) or
// [`App::new`](crate::app::App::new).
pub struct Surface {
    lib: Arc<Library>,
    internal: ul_sys::ULSurface,
}

impl Surface {
    /// Helper internal function to allow getting a reference to a managed
    /// surface.
    pub(crate) unsafe fn from_raw(lib: Arc<Library>, raw: ul_sys::ULSurface) -> Self {
        Self { lib, internal: raw }
    }
}

impl Surface {
    /// Get the width (in pixels).
    pub fn width(&self) -> u32 {
        unsafe { self.lib.ultralight().ulSurfaceGetWidth(self.internal) }
    }

    /// Get the height (in pixels).
    pub fn height(&self) -> u32 {
        unsafe { self.lib.ultralight().ulSurfaceGetHeight(self.internal) }
    }

    /// Get the number of bytes between each row of pixels.
    ///
    /// usually `width * 4`
    pub fn row_bytes(&self) -> u32 {
        unsafe { self.lib.ultralight().ulSurfaceGetRowBytes(self.internal) }
    }

    /// Get the size in bytes of the pixel buffer.
    ///
    /// bytes_size is calculated as `row_bytes() * height()`.
    pub fn bytes_size(&self) -> usize {
        unsafe { self.lib.ultralight().ulSurfaceGetSize(self.internal) }
    }

    /// Lock the pixel buffer for reading/writing.
    ///
    /// An RAII guard is returned that will unlock the buffer when dropped.
    //
    // this takes `&mut` even though its not needed to lock the structure,
    // so that you can't resize or modify while its locked.
    pub fn lock_pixels(&mut self) -> Option<PixelsGuard> {
        let raw_locked_pixels = unsafe { self.lib.ultralight().ulSurfaceLockPixels(self.internal) };
        if raw_locked_pixels.is_null() {
            return None;
        }

        let size = self.bytes_size();
        unsafe {
            let data = std::slice::from_raw_parts_mut(raw_locked_pixels as _, size);
            Some(PixelsGuard::new(self, data))
        }
    }

    /// Internal unlock the pixel buffer.
    pub(crate) unsafe fn raw_unlock_pixels(&self) {
        self.lib.ultralight().ulSurfaceUnlockPixels(self.internal)
    }

    /// Resize the pixel buffer to a certain width and height (both in pixels).
    pub fn resize(&self, width: u32, height: u32) {
        unsafe {
            self.lib
                .ultralight()
                .ulSurfaceResize(self.internal, width, height)
        }
    }

    /// Set the dirty bounds to a certain value.
    ///
    /// This is called after the Renderer paints to an area of the pixel buffer.
    /// (The new value will be joined with the existing dirty_bounds())
    pub fn set_dirty_bounds(&self, bounds: Rect<i32>) {
        unsafe {
            self.lib
                .ultralight()
                .ulSurfaceSetDirtyBounds(self.internal, bounds.into())
        }
    }

    /// Get the dirty bounds.
    ///
    /// This value can be used to determine which portion of the pixel buffer has been updated since
    /// the last call to [`clear_dirty_bounds`](Surface::clear_dirty_bounds).
    ///
    /// The general algorithm to determine if a Surface needs display is:
    /// ```rust,ignore
    /// if !surface.dirty_bounds().is_empty() {
    ///     // Surface pixels are dirty and needs display.
    ///     // Cast Surface to native Surface and use it here (pseudo code)
    ///     display_surface(surface);
    ///
    ///     // Once you're done, clear the dirty bounds:
    ///     surface.clear_dirty_bounds();
    /// }
    /// ```
    pub fn dirty_bounds(&self) -> Rect<i32> {
        unsafe {
            self.lib
                .ultralight()
                .ulSurfaceGetDirtyBounds(self.internal)
                .into()
        }
    }

    /// Clear the dirty bounds.
    ///
    /// You should call this after you're done displaying the Surface.
    pub fn clear_dirty_bounds(&self) {
        unsafe {
            self.lib
                .ultralight()
                .ulSurfaceClearDirtyBounds(self.internal)
        }
    }

    //pub fn user_data(&self) -> *mut std::ffi::c_void {
    //    unsafe { ul_sys::ulSurfaceGetUserData(self.internal) }
    //}
}
