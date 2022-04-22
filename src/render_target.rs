use crate::rect::Rect;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum BitmapFormat {
    ///
    /// Alpha channel only, 8-bits per pixel.
    ///
    /// Encoding: 8-bits per channel, unsigned normalized.
    ///
    /// Color-space: Linear (no gamma), alpha-coverage only.
    ///
    A8_UNORM = ul_sys::ULBitmapFormat_kBitmapFormat_A8_UNORM as isize,

    ///
    /// Blue Green Red Alpha channels, 32-bits per pixel.
    ///
    /// Encoding: 8-bits per channel, unsigned normalized.
    ///
    /// Color-space: sRGB gamma with premultiplied linear alpha channel.
    ///
    BGRA8_UNORM_SRGB = ul_sys::ULBitmapFormat_kBitmapFormat_BGRA8_UNORM_SRGB as isize,
}

impl TryFrom<ul_sys::ULBitmapFormat> for BitmapFormat {
    // TODO: handle errors
    type Error = ();

    fn try_from(value: ul_sys::ULBitmapFormat) -> Result<Self, Self::Error> {
        match value {
            ul_sys::ULBitmapFormat_kBitmapFormat_A8_UNORM => Ok(BitmapFormat::A8_UNORM),
            ul_sys::ULBitmapFormat_kBitmapFormat_BGRA8_UNORM_SRGB => {
                Ok(BitmapFormat::BGRA8_UNORM_SRGB)
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderTarget {
    pub is_empty: bool,
    pub width: u32,
    pub height: u32,
    pub texture_id: u32,
    pub texture_width: u32,
    pub texture_height: u32,
    pub texture_format: BitmapFormat,
    pub uv_coords: Rect<f32>,
    pub render_buffer_id: u32,
}

impl From<ul_sys::ULRenderTarget> for RenderTarget {
    fn from(rt: ul_sys::ULRenderTarget) -> Self {
        RenderTarget {
            is_empty: rt.is_empty,
            width: rt.width,
            height: rt.height,
            texture_id: rt.texture_id,
            texture_width: rt.texture_width,
            texture_height: rt.texture_height,
            texture_format: BitmapFormat::try_from(rt.texture_format).unwrap(),
            uv_coords: rt.uv_coords.into(),
            render_buffer_id: rt.render_buffer_id,
        }
    }
}
