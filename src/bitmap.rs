//! Bitmap container to hold raw pixels data.

use std::{
    ffi::{c_void, CString},
    ops::{Deref, DerefMut},
    path::Path,
    slice,
    sync::Arc,
};

use crate::Library;

/// Errors can occure when creating [`Bitmap`]s
#[derive(Debug, thiserror::Error)]
pub enum BitmapError {
    /// The `Ultralight` library returned a null pointer.
    #[error("Creation of bitmap failed because Ultralight returned a null pointer")]
    NullReference,
    /// The pixels passed to create the [`Bitmap`] does not match the required size.
    #[error(
        "Creation of bitmap failed because it required {required} bytes, but got {got} bytes only"
    )]
    PixelBufferSizeMismatch { got: usize, required: usize },
    /// Tried to swap red and blue channels on an unsupported format.
    #[error("Tried to swap red and blue channels on an unsupported format")]
    UnsupportedOperationForPixelFormat,
    /// Could not write bitmap to PNG successfully.
    #[error("Could not write bitmap to PNG successfully")]
    FailedPngWrite,
    /// Could not create bitmap because its empty
    #[error("Could not create bitmap because its empty")]
    EmptyBitmap,
}

type BitmapResult<T> = std::result::Result<T, BitmapError>;

#[derive(Debug, Clone, Copy)]
/// The supported bitmap formats.
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
    /// Returns the number of bytes per pixel for the specific bitmap format.
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            BitmapFormat::A8Unorm => 1,
            BitmapFormat::Bgra8UnormSrgb => 4,
        }
    }
}

/// An RAII implementation of a “scoped lock” of a pixel buffer for [`Bitmap`].
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
///
/// This struct is created by [`Bitmap::lock_pixels`].
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

/// `Ultralight` Bitmap container.
pub struct Bitmap {
    lib: Arc<Library>,
    internal: ul_sys::ULBitmap,
    need_to_destroy: bool,
}

impl Bitmap {
    pub(crate) unsafe fn from_raw(lib: Arc<Library>, raw: ul_sys::ULBitmap) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        Some(Bitmap {
            lib,
            internal: raw,
            need_to_destroy: false,
        })
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULBitmap {
        self.internal
    }

    /// Create an empty Bitmap. No pixels will be allocated.
    pub fn create_empty(lib: Arc<Library>) -> BitmapResult<Self> {
        let internal = unsafe { lib.ultralight().ulCreateEmptyBitmap() };

        if internal.is_null() {
            Err(BitmapError::NullReference)
        } else {
            Ok(Self {
                lib,
                internal,
                need_to_destroy: true,
            })
        }
    }

    /// Create an aligned Bitmap with a certain configuration. Pixels will be allocated but not
    /// initialized.
    ///
    /// # Arguments
    /// * `lib` - The ultralight library.
    /// * `width` - The width of the bitmap.
    /// * `height` - The height of the bitmap.
    /// * `format` - The format of the bitmap.
    pub fn create(
        lib: Arc<Library>,
        width: usize,
        height: usize,
        format: BitmapFormat,
    ) -> BitmapResult<Self> {
        let internal = unsafe {
            lib.ultralight()
                .ulCreateBitmap(width as u32, height as u32, format as u32)
        };
        if internal.is_null() {
            Err(BitmapError::NullReference)
        } else {
            Ok(Self {
                lib,
                internal,
                need_to_destroy: true,
            })
        }
    }

    /// Create a Bitmap with existing pixels
    ///
    /// # Arguments
    /// * `lib` - The ultralight library.
    /// * `width` - The width of the bitmap.
    /// * `height` - The height of the bitmap.
    /// * `format` - The format of the bitmap.
    /// * `pixels` - The raw pixels of the bitmap.
    ///
    /// The length of the `pixels` slice must be equal to `width * height * format.bytes_per_pixel()`.
    pub fn create_from_pixels(
        lib: Arc<Library>,
        width: u32,
        height: u32,
        format: BitmapFormat,
        pixels: &[u8],
    ) -> BitmapResult<Self> {
        let row_bytes = width * format.bytes_per_pixel();
        let bytes_size = (height * row_bytes) as usize;
        if pixels.len() != bytes_size {
            return Err(BitmapError::PixelBufferSizeMismatch {
                got: pixels.len(),
                required: bytes_size,
            });
        }
        // This will create a new buffer and copy the pixels into it
        // NOTE: the constructor allow for row size that is more than the actual row size
        //       which means that it can support padding, we don't need it for now
        //       but if needed, we can implement it.
        let internal = unsafe {
            lib.ultralight().ulCreateBitmapFromPixels(
                width,
                height,
                format as u32,
                row_bytes,
                pixels.as_ptr() as *const c_void,
                pixels.len(),
                true,
            )
        };
        if internal.is_null() {
            Err(BitmapError::NullReference)
        } else {
            Ok(Self {
                lib,
                internal,
                need_to_destroy: true,
            })
        }
    }

    /// Create a bitmap from a deep copy of another Bitmap.
    pub fn copy(&self) -> BitmapResult<Self> {
        let internal = unsafe { self.lib.ultralight().ulCreateBitmapFromCopy(self.internal) };

        if internal.is_null() {
            Err(BitmapError::NullReference)
        } else {
            Ok(Self {
                lib: self.lib.clone(),
                internal,
                need_to_destroy: true,
            })
        }
    }
}

impl Bitmap {
    /// Get the width in pixels.
    pub fn width(&self) -> u32 {
        unsafe { self.lib.ultralight().ulBitmapGetWidth(self.internal) }
    }

    /// Get the height in pixels.
    pub fn height(&self) -> u32 {
        unsafe { self.lib.ultralight().ulBitmapGetHeight(self.internal) }
    }

    /// Get the pixel format.
    pub fn format(&self) -> BitmapFormat {
        unsafe { self.lib.ultralight().ulBitmapGetFormat(self.internal) }
            .try_into()
            .unwrap()
    }

    /// Get the number of bytes per pixel.
    pub fn bpp(&self) -> u32 {
        unsafe { self.lib.ultralight().ulBitmapGetBpp(self.internal) }
    }

    /// Get the number of bytes between each row of pixels.
    ///
    /// This value is usually calculated as `width() * bytes_per_pixel()` (bpp) but it may be larger
    /// due to alignment rules in the allocator.
    pub fn row_bytes(&self) -> u32 {
        unsafe { self.lib.ultralight().ulBitmapGetRowBytes(self.internal) }
    }

    /// Get the size in bytes of the pixel buffer.
    ///
    /// bytes_size is calculated as `row_bytes() * height()`.
    pub fn bytes_size(&self) -> usize {
        unsafe { self.lib.ultralight().ulBitmapGetSize(self.internal) }
    }

    /// Lock the pixel buffer for reading/writing.
    ///
    /// An RAII guard is returned that will unlock the buffer when dropped.
    pub fn lock_pixels(&mut self) -> Option<PixelsGuard> {
        let (raw_pixels, size) = unsafe {
            self.lib.ultralight().ulBitmapLockPixels(self.internal);
            (
                self.lib.ultralight().ulBitmapRawPixels(self.internal),
                self.lib.ultralight().ulBitmapGetSize(self.internal),
            )
        };

        if raw_pixels.is_null() {
            return None;
        }

        unsafe {
            let data = slice::from_raw_parts_mut(raw_pixels as _, size);
            Some(PixelsGuard::new(self, data))
        }
    }

    /// Internal unlock the pixel buffer.
    pub(crate) unsafe fn raw_unlock_pixels(&mut self) {
        self.lib.ultralight().ulBitmapUnlockPixels(self.internal);
    }

    /// Whether or not this bitmap is empty (no pixels allocated).
    pub fn is_empty(&self) -> bool {
        unsafe { self.lib.ultralight().ulBitmapIsEmpty(self.internal) }
    }

    /// Reset bitmap pixels to 0.
    pub fn erase(&self) {
        unsafe { self.lib.ultralight().ulBitmapErase(self.internal) }
    }

    /// Write bitmap to a PNG on disk.
    pub fn write_to_png<P: AsRef<Path>>(&self, path: P) -> BitmapResult<()> {
        let c_path = CString::new(path.as_ref().to_str().unwrap()).unwrap();
        let result = unsafe {
            self.lib
                .ultralight()
                .ulBitmapWritePNG(self.internal, c_path.as_ptr())
        };
        if result {
            Ok(())
        } else {
            Err(BitmapError::FailedPngWrite)
        }
    }

    /// This converts a BGRA bitmap to RGBA bitmap and vice-versa by swapping the red and blue channels.
    ///
    /// Only valid if the format is BitmapFormat::BGRA8_UNORM_SRGB
    pub fn swap_red_blue_channels(&self) -> BitmapResult<()> {
        if let BitmapFormat::Bgra8UnormSrgb = self.format() {
            unsafe {
                self.lib
                    .ultralight()
                    .ulBitmapSwapRedBlueChannels(self.internal)
            }
            Ok(())
        } else {
            Err(BitmapError::UnsupportedOperationForPixelFormat)
        }
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe { self.lib.ultralight().ulDestroyBitmap(self.internal) };
        }
    }
}

/// A bitmap object that has an owned pixel buffer.
///
/// This is useful for using the raw pixels in any rust code without
/// binding to the underlying C library.
///
/// To create an `Ultralight` bitmap, use [`OwnedBitmap::to_bitmap`].
pub struct OwnedBitmap {
    width: u32,
    height: u32,
    format: BitmapFormat,
    bpp: u32,
    row_bytes: u32,
    bytes_size: usize,
    pixels: Option<Vec<u8>>,
    is_empty: bool,
}

impl OwnedBitmap {
    /// Create an [`OwnedBitmap`] from a [`Bitmap`].
    ///
    /// This will result in copying all the pixels from the original bitmap.
    pub fn from_bitmap(bitmap: &mut Bitmap) -> Option<Self> {
        let width = bitmap.width();
        let height = bitmap.height();
        let format = bitmap.format();
        let bpp = bitmap.bpp();
        let row_bytes = bitmap.row_bytes();
        let bytes_size = bitmap.bytes_size();
        let is_empty = bitmap.is_empty();

        let pixels = bitmap.lock_pixels().map(|v| v.to_vec());

        Some(Self {
            width,
            height,
            format,
            bpp,
            row_bytes,
            bytes_size,
            pixels,
            is_empty,
        })
    }

    /// Create a [`Bitmap`] from an [`OwnedBitmap`].
    ///
    /// This is useful when we need to call `Ultralight` logic that require [`Bitmap`].
    ///
    /// This function will copy all the pixels from the owned bitmap.
    pub fn to_bitmap(&self, lib: Arc<Library>) -> BitmapResult<Bitmap> {
        if let Some(pixels) = self.pixels.as_ref() {
            Bitmap::create_from_pixels(lib, self.width, self.height, self.format, pixels.as_slice())
        } else {
            Err(BitmapError::EmptyBitmap)
        }
    }

    /// Get the width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the pixel format.
    pub fn format(&self) -> BitmapFormat {
        self.format
    }

    /// Get the number of bytes per pixel.
    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    /// Get the number of bytes between each row of pixels.
    pub fn row_bytes(&self) -> u32 {
        self.row_bytes
    }

    /// Get the size in bytes of the pixel buffer.
    pub fn bytes_size(&self) -> usize {
        self.bytes_size
    }

    /// Get the pixel buffer slice.
    pub fn pixels(&self) -> Option<&[u8]> {
        self.pixels.as_deref()
    }

    /// Get the mutable pixel buffer slice.
    pub fn pixels_mut(&mut self) -> Option<&mut [u8]> {
        self.pixels.as_deref_mut()
    }

    /// Whether or not this bitmap is empty (no pixels allocated).
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }
}
