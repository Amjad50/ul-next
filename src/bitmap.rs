use std::{
    ffi::{c_void, CString},
    ops::{Deref, DerefMut},
    path::Path,
    slice,
};

pub enum BitmapFormat {
    /// Alpha channel only, 8-bits per pixel.
    ///
    /// Encoding: 8-bits per channel, unsigned normalized.
    ///
    /// Color-space: Linear (no gamma), alpha-coverage only.
    A8Unorm = ul_sys::ULBitmapFormat_kBitmapFormat_A8_UNORM as isize,

    /// Blue Green Red Alpha channels, 32-bits per pixel.
    ///
    /// Encoding: 8-bits per channel, unsigned normalized.
    ///
    /// Color-space: sRGB gamma with premultiplied linear alpha channel.
    Bgra8UnormSrgb = ul_sys::ULBitmapFormat_kBitmapFormat_BGRA8_UNORM_SRGB as isize,
}

impl TryFrom<ul_sys::ULBitmapFormat> for BitmapFormat {
    type Error = ();

    fn try_from(format: ul_sys::ULBitmapFormat) -> Result<Self, Self::Error> {
        match format {
            ul_sys::ULBitmapFormat_kBitmapFormat_A8_UNORM => Ok(BitmapFormat::A8Unorm),
            ul_sys::ULBitmapFormat_kBitmapFormat_BGRA8_UNORM_SRGB => {
                Ok(BitmapFormat::Bgra8UnormSrgb)
            }
            _ => Err(()),
        }
    }
}

impl BitmapFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            BitmapFormat::A8Unorm => 1,
            BitmapFormat::Bgra8UnormSrgb => 4,
        }
    }
}

pub struct PixelsGuard<'a> {
    lock: &'a mut Bitmap,
    pixels: &'a mut [u8],
}

impl<'a> PixelsGuard<'a> {
    unsafe fn new(lock: &'a mut Bitmap, pixels: &'a mut [u8]) -> PixelsGuard<'a> {
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

pub struct Bitmap {
    internal: ul_sys::ULBitmap,
    need_to_destroy: bool,
}

impl Bitmap {
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULBitmap) -> Self {
        Bitmap {
            internal: raw,
            need_to_destroy: false,
        }
    }

    /// Create an empty Bitmap. No pixels will be allocated.
    pub fn create_empty() -> Self {
        Self {
            internal: unsafe { ul_sys::ulCreateEmptyBitmap() },
            need_to_destroy: true,
        }
    }

    /// Create an aligned Bitmap with a certain configuration. Pixels will be allocated but not
    /// initialized.
    pub fn create(width: usize, height: usize, format: BitmapFormat) -> Self {
        Self {
            internal: unsafe { ul_sys::ulCreateBitmap(width as u32, height as u32, format as u32) },
            need_to_destroy: true,
        }
    }

    /// Create a Bitmap with existing pixels
    pub fn create_from_pixels(
        width: u32,
        height: u32,
        format: BitmapFormat,
        pixels: &[u8],
    ) -> Self {
        let row_bytes = width * format.bytes_per_pixel();
        assert!(pixels.len() == (row_bytes * height) as usize);
        Self {
            // This will create a new buffer and copy the pixels into it
            // NOTE: the constructor allow for row size that is more than the actual row size
            //       which means that it can support padding, we don't need it for now
            //       but if needed, we can implement it.
            internal: unsafe {
                ul_sys::ulCreateBitmapFromPixels(
                    width,
                    height,
                    format as u32,
                    row_bytes.try_into().unwrap(),
                    pixels.as_ptr() as *const c_void,
                    pixels.len() as u64,
                    true,
                )
            },
            need_to_destroy: true,
        }
    }

    /// Create a bitmap from a deep copy of another Bitmap.
    pub fn copy(&self) -> Self {
        Self {
            internal: unsafe { ul_sys::ulCreateBitmapFromCopy(self.internal) },
            need_to_destroy: true,
        }
    }
}

impl Bitmap {
    /// Get the width in pixels.
    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulBitmapGetWidth(self.internal) }
    }

    /// Get the height in pixels.
    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulBitmapGetHeight(self.internal) }
    }

    /// Get the pixel format.
    pub fn format(&self) -> BitmapFormat {
        unsafe { ul_sys::ulBitmapGetFormat(self.internal) }
            .try_into()
            .unwrap()
    }

    /// Get the number of bytes per pixel.
    pub fn bpp(&self) -> u32 {
        unsafe { ul_sys::ulBitmapGetBpp(self.internal) }
    }

    /// Get the number of bytes between each row of pixels.
    ///
    /// This value is usually calculated as width * bytes_per_pixel (bpp) but it may be larger
    /// due to alignment rules in the allocator.
    pub fn row_bytes(&self) -> u32 {
        unsafe { ul_sys::ulBitmapGetRowBytes(self.internal) }
    }

    /// Get the size in bytes of the pixel buffer.
    ///
    /// bytes_size is calculated as row_bytes() * height().
    pub fn bytes_size(&self) -> u64 {
        unsafe { ul_sys::ulBitmapGetSize(self.internal) }
    }

    pub fn lock_pixels(&mut self) -> PixelsGuard {
        let (raw_pixels, size) = unsafe {
            ul_sys::ulBitmapLockPixels(self.internal);
            (
                ul_sys::ulBitmapRawPixels(self.internal),
                ul_sys::ulBitmapGetSize(self.internal),
            )
        };

        unsafe {
            let data = slice::from_raw_parts_mut(raw_pixels as _, size as usize);
            PixelsGuard::new(self, data)
        }
    }

    pub(crate) unsafe fn raw_unlock_pixels(&mut self) {
        ul_sys::ulBitmapUnlockPixels(self.internal);
    }

    /// Whether or not this bitmap is empty (no pixels allocated).
    pub fn is_empty(&self) -> bool {
        unsafe { ul_sys::ulBitmapIsEmpty(self.internal) }
    }

    /// Reset bitmap pixels to 0.
    pub fn erase(&self) {
        unsafe { ul_sys::ulBitmapErase(self.internal) }
    }

    /// Write bitmap to a PNG on disk.
    pub fn write_to_png<P: AsRef<Path>>(&self, path: P) -> Result<(), ()> {
        let c_path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
        let result = unsafe { ul_sys::ulBitmapWritePNG(self.internal, c_path.as_ptr()) };
        if result {
            Ok(())
        } else {
            Err(())
        }
    }

    /// This converts a BGRA bitmap to RGBA bitmap and vice-versa by swapping the red and blue channels.
    ///
    /// Only valid if the format is BitmapFormat::BGRA8_UNORM_SRGB
    pub fn swap_red_blue_channels(&self) -> Result<(), ()> {
        if let BitmapFormat::Bgra8UnormSrgb = self.format() {
            unsafe { ul_sys::ulBitmapSwapRedBlueChannels(self.internal) }
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe { ul_sys::ulDestroyBitmap(self.internal) };
        }
    }
}
