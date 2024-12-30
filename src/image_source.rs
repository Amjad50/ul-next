//! User-defined image source to display custom images on a web-page.

use std::sync::Arc;

use crate::{bitmap::Bitmap, error::CreationError, string::UlString, Library, Rect};

/// User-defined image source to display custom images on a web-page.
///
/// This API allows you to composite your own images into a web-page. This is useful for displaying
/// in-game textures, external image assets, or other custom content.
///
/// ## ImageSource File Format
///
/// To use an ImageSource, you must first create an `.imgsrc` file containing a string identifying
/// the image source. This string will be used to lookup the ImageSource from ImageSourceProvider
/// when it is loaded on a web-page.
///
/// The file format is as follows:
///
/// ```txt
/// IMGSRC-V1
/// <identifier>
/// ```
///
/// You can use the `.imgsrc` file anywhere in your web-page that typically accepts an image URL.
/// For example:
///
/// ```html
/// <img src="my_custom_image.imgsrc" />
/// ```
///
/// ## Creating from a GPU Texture
///
/// To composite your own GPU texture on a web-page, you should first reserve a texture ID from
/// [`GpuDriver::next_texture_id`][crate::gpu_driver::GpuDriver::next_texture_id] using
/// and then create an ImageSource from that texture ID
/// using [`ImageSource::create_from_texture`][ImageSource::create_from_texture].
///
/// Next, you should register the [`ImageSource`] with [`image_source_provider::add_image_source`]
/// using the identifier from the `.imgsrc` file.
///
/// When the image element is drawn on the web-page, the library will draw geometry using the
/// specified texture ID and UV coordinates. You should bind your own texture when the specified
/// texture ID is used.
///
/// Note: If the GPU renderer is not enabled for the View or pixel data is needed for other
///       purposes, the library will sample the backing bitmap instead.
///
/// ## Creating from a Bitmap
///
/// To composite your own bitmap on a web-page, you should create an [`ImageSource`] from a bitmap
/// using [`ImageSource::create_from_bitmap`][ImageSource::create_from_bitmap].
///
/// Next, you should register the [`ImageSource`] with [`image_source_provider::add_image_source`]
/// using the identifier from the `.imgsrc` file.
///
/// When the image element is drawn on the web-page, the library will sample this bitmap directly.
///
/// ## Invalidating Images
///
/// If you modify the texture or bitmap pixels after creating the ImageSource, you should call
/// [`ImageSource::invalidate`] to notify the library that the image should be redrawn.
pub struct ImageSource {
    lib: Arc<Library>,
    internal: ul_sys::ULImageSource,
}

impl ImageSource {
    /// Create an image source from a GPU texture with optional backing bitmap.
    /// # Arguments
    /// * `lib` - The ultralight library.
    /// * `width` - The width of the texture in pixels (used for layout).
    /// * `height` - The height of the texture in pixels (used for layout).
    /// * `texture_id` - The GPU texture identifier to bind when drawing the quad for this image.
    ///                  This should be non-zero and obtained from
    ///                  [`GpuDriver::next_texture_id`][crate::gpu_driver::GpuDriver::next_texture_id].
    /// * `rect` - The rectangle in UV coordinates to sample from the texture.
    /// * `bitmap` - Optional backing bitmap for the texture. This is used when drawing
    ///              the image using the CPU renderer or when pixel data is needed for other
    ///              purposes. You should update this bitmap when the texture changes.
    pub fn create_from_texture(
        lib: Arc<Library>,
        width: u32,
        height: u32,
        texture_id: u32,
        rect: Rect<f32>,
        bitmap: Option<Bitmap>,
    ) -> Result<ImageSource, CreationError> {
        let internal = unsafe {
            lib.ultralight().ulCreateImageSourceFromTexture(
                width,
                height,
                texture_id,
                ul_sys::ULRect {
                    left: rect.left,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                },
                bitmap.map(|b| b.to_ul()).unwrap_or(std::ptr::null_mut()),
            )
        };
        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { lib, internal })
        }
    }

    /// Create an image source from a bitmap.
    /// # Arguments
    /// * `lib` - The ultralight library.
    /// * `bitmap` - The bitmap to sample from when drawing the image.
    pub fn create_from_bitmap(
        lib: Arc<Library>,
        bitmap: Bitmap,
    ) -> Result<ImageSource, CreationError> {
        let internal = unsafe {
            lib.ultralight()
                .ulCreateImageSourceFromBitmap(bitmap.to_ul())
        };
        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { lib, internal })
        }
    }

    /// Invalidate the image source, notifying the library that the image has changed
    /// and should be redrawn
    pub fn invalidate(&self) {
        unsafe {
            self.lib.ultralight().ulImageSourceInvalidate(self.internal);
        }
    }
}

impl Drop for ImageSource {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().ulDestroyImageSource(self.internal);
        }
    }
}

/// Maps image sources to string identifiers.
///
/// This is used to lookup ImageSource instances when they are requested by a web-page.
pub mod image_source_provider {
    use crate::Library;
    use std::sync::Arc;

    /// Add an image source to the provider.
    pub fn add_image_source(
        id: &str,
        image_source: &super::ImageSource,
    ) -> Result<(), super::CreationError> {
        unsafe {
            let id_str = super::UlString::from_str(image_source.lib.clone(), id)?;
            image_source
                .lib
                .ultralight()
                .ulImageSourceProviderAddImageSource(id_str.to_ul(), image_source.internal);
        }
        Ok(())
    }

    /// Remove an image source from the provider.
    pub fn remove_image_source(lib: &Arc<Library>, id: &str) -> Result<(), super::CreationError> {
        unsafe {
            let id_str = super::UlString::from_str(lib.clone(), id)?;
            lib.ultralight()
                .ulImageSourceProviderRemoveImageSource(id_str.to_ul());
        }
        Ok(())
    }
}
