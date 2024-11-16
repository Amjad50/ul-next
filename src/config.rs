//! The configuration of the [`Renderer`](crate::renderer::Renderer) struct.

use std::sync::Arc;

use crate::Library;

/// The winding order for front-facing triangles. (Only used when the GPU renderer is used)
pub enum FaceWinding {
    /// Clockwise Winding (Direct3D, etc.)
    Clockwise = ul_sys::ULFaceWinding_kFaceWinding_Clockwise as isize,
    /// Counter-Clockwise Winding (OpenGL, etc.)
    CounterClockwise = ul_sys::ULFaceWinding_kFaceWinding_CounterClockwise as isize,
}

/// The font hinting algorithm.
pub enum FontHinting {
    /// Lighter hinting algorithm-- glyphs are slightly fuzzier but better resemble their original
    /// shape. This is achieved by snapping glyphs to the pixel grid only vertically which better
    /// preserves inter-glyph spacing.
    Smooth = ul_sys::ULFontHinting_kFontHinting_Smooth as isize,
    /// Default hinting algorithm-- offers a good balance between sharpness and shape at smaller font
    /// sizes.
    Normal = ul_sys::ULFontHinting_kFontHinting_Normal as isize,
    /// Strongest hinting algorithm-- outputs only black/white glyphs. The result is usually
    /// unpleasant if the underlying TTF does not contain hints for this type of rendering.
    Monochrome = ul_sys::ULFontHinting_kFontHinting_Monochrome as isize,
}

/// Configuration settings for Ultralight renderer
///
/// This is intended to be implemented by users when creating the Renderer in
/// [`Renderer::create`](crate::renderer::Renderer::create).
pub struct Config {
    lib: Arc<Library>,
    internal: ul_sys::ULConfig,
}

impl Config {
    /// Starts the building process for the [`Config`] struct. returns a builder
    /// which can be used to configure the settings.
    pub fn start() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Returns the underlying [`ul_sys::ULConfig`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULConfig {
        self.internal
    }

    /// Returns the library associated with this config.
    pub(crate) fn lib(&self) -> &Arc<Library> {
        &self.lib
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().ulDestroyConfig(self.internal);
        }
    }
}

/// Builder for the [`Config`] struct.
#[derive(Default)]
pub struct ConfigBuilder {
    cache_path: Option<String>,
    resource_path_prefix: Option<String>,
    face_winding: Option<FaceWinding>,
    font_hinting: Option<FontHinting>,
    font_gamma: Option<f64>,
    user_stylesheet: Option<String>,
    force_repaint: Option<bool>,
    animation_timer_delay: Option<f64>,
    scroll_timer_delay: Option<f64>,
    recycle_delay: Option<f64>,
    memory_cache_size: Option<u32>,
    page_cache_size: Option<u32>,
    override_ram_size: Option<u32>,
    min_large_heap_size: Option<u32>,
    min_small_heap_size: Option<u32>,
    num_renderer_threads: Option<u32>,
    max_update_time: Option<f64>,
    bitmap_alignment: Option<u32>,
}

impl ConfigBuilder {
    /// A writable OS file path to store persistent Session data in.
    ///
    /// This data may include cookies, cached network resources, indexed DB, etc.
    ///
    /// Files are only written to disk when using a persistent Session (see
    /// [`Renderer::create_session`](crate::renderer::Renderer::create_session)).
    pub fn cache_path(mut self, path: &str) -> Self {
        self.cache_path = Some(path.to_string());
        self
    }

    /// The relative path to the resources folder (loaded via the FileSystem API).
    /// The library loads certain resources (SSL certs, ICU data, etc.)
    /// from the FileSystem API during runtime (eg, `file:///resources/cacert.pem`).
    ///
    /// You can customize the relative file path to the resources folder by modifying this setting.
    ///
    /// (Default = “resources/”)
    pub fn resource_path_prefix(mut self, path: &str) -> Self {
        self.resource_path_prefix = Some(path.to_string());
        self
    }

    /// The winding order for front-facing triangles. (See [`FaceWinding`])
    ///
    /// Note: This is only used when the GPU renderer is enabled.
    pub fn face_winding(mut self, winding: FaceWinding) -> Self {
        self.face_winding = Some(winding);
        self
    }

    /// The hinting algorithm to use when rendering fonts. (See [`FontHinting`])
    pub fn font_hinting(mut self, hinting: FontHinting) -> Self {
        self.font_hinting = Some(hinting);
        self
    }

    /// The gamma to use when compositing font glyphs, change this value to
    /// adjust contrast (Adobe and Apple prefer 1.8, others may prefer 2.2).
    pub fn font_gamma(mut self, gamma: f64) -> Self {
        self.font_gamma = Some(gamma);
        self
    }

    /// Default user stylesheet. You should set this to your own custom CSS
    /// string to define default styles for various DOM elements, scrollbars,
    /// and platform input widgets.
    pub fn user_stylesheet(mut self, path: &str) -> Self {
        self.user_stylesheet = Some(path.to_string());
        self
    }

    /// Whether or not we should continuously repaint any Views or compositor
    /// layers, regardless if they are dirty or not. This is mainly used to
    /// diagnose painting/shader issues.
    ///
    /// (Default = false)
    pub fn force_repaint(mut self, force: bool) -> Self {
        self.force_repaint = Some(force);
        self
    }

    /// When a CSS animation is active, the amount of time (in seconds) to wait
    /// before triggering another repaint.
    ///
    /// (Default = 1.0 / 60.0)
    pub fn animation_timer_delay(mut self, delay: f64) -> Self {
        self.animation_timer_delay = Some(delay);
        self
    }

    /// When a smooth scroll animation is active, the amount of time (in seconds)
    /// to wait before triggering another repaint.
    ///
    /// (Default = 1.0 / 60.0)
    pub fn scroll_timer_delay(mut self, delay: f64) -> Self {
        self.scroll_timer_delay = Some(delay);
        self
    }

    /// The amount of time (in seconds) to wait before running the recycler
    /// (will attempt to return excess memory back to the system).
    pub fn recycle_delay(mut self, delay: f64) -> Self {
        self.recycle_delay = Some(delay);
        self
    }

    /// The size of WebCore's memory cache in bytes.
    ///
    /// You should increase this if you anticipate handling pages with large
    /// resources, Safari typically uses 128+ MiB for its cache.
    ///
    /// `size` is in bytes.
    ///
    /// (Default = 64 * 1024 * 1024)
    pub fn memory_cache_size(mut self, size: u32) -> Self {
        self.memory_cache_size = Some(size);
        self
    }

    /// Number of pages to keep in the cache. Defaults to 0 (none).
    ///
    /// Safari typically caches about 5 pages and maintains an on-disk cache
    /// to support typical web-browsing activities. If you increase this,
    /// you should probably increase the memory cache size as well.
    pub fn page_cache_size(mut self, size: u32) -> Self {
        self.page_cache_size = Some(size);
        self
    }

    /// The system's physical RAM size in bytes.
    ///
    /// JavaScriptCore tries to detect the system's physical RAM size to set
    /// reasonable allocation limits. Set this to anything other than 0 to
    /// override the detected value. `size` is in bytes.
    ///
    /// This can be used to force JavaScriptCore to be more conservative
    /// with its allocation strategy (at the cost of some performance).
    pub fn override_ram_size(mut self, size: u32) -> Self {
        self.override_ram_size = Some(size);
        self
    }

    /// The minimum size of large VM heaps in JavaScriptCore. Set this to a lower value to make these
    /// heaps start with a smaller initial value.
    ///
    /// `size` is in bytes.
    ///
    /// (Default = 32 * 1024 * 1024)
    pub fn min_large_heap_size(mut self, size: u32) -> Self {
        self.min_large_heap_size = Some(size);
        self
    }

    /// The minimum size of small VM heaps in JavaScriptCore. Set this to a lower value to make these
    /// heaps start with a smaller initial value.
    ///
    /// `size` is in bytes.
    ///
    /// (Default = 1 * 1024 * 1024)
    pub fn min_small_heap_size(mut self, size: u32) -> Self {
        self.min_small_heap_size = Some(size);
        self
    }

    /// The number of threads to use in the Renderer (for parallel painting on the CPU, etc.).
    ///
    /// You can set this to a certain number to limit the number of threads to spawn.
    ///
    /// If this value is 0 (the default), the number of threads will be determined at runtime
    /// using the following formula:
    ///
    /// `max(PhysicalProcessorCount() - 1, 1)`
    pub fn num_renderer_threads(mut self, threads: u32) -> Self {
        self.num_renderer_threads = Some(threads);
        self
    }

    /// The max amount of time (in seconds) to allow repeating timers to run during each call to
    /// [`Renderer::update`](crate::renderer::Renderer::update).
    /// The library will attempt to throttle timers and/or reschedule work if this
    /// time budget is exceeded.
    ///
    /// (Default = 1.0 / 200.0)
    pub fn max_update_time(mut self, time: f64) -> Self {
        self.max_update_time = Some(time);
        self
    }

    /// **Note that this is currently is useless in this library**
    /// **as we can't get the bitmap from [`Surface`](crate::surface::Surface)**
    /// **when using CPU rendering**.
    ///
    /// The alignment (in bytes) of the BitmapSurface when using the CPU renderer.
    ///
    /// The underlying bitmap associated with each BitmapSurface will have
    /// `row_bytes` padded to reach this `alignment`.
    ///
    /// Aligning the bitmap helps improve performance when using the CPU renderer/
    /// Determining the proper value to use depends on the CPU architecture and
    /// max SIMD instruction set used.
    ///
    /// We generally target the 128-bit SSE2 instruction set across most
    /// PC platforms so '16' is a default and safe value to use.
    ///
    /// You can set this to '0' to perform no padding
    /// (row_bytes will always be `width * 4`) at a slight cost to performance.
    pub fn bitmap_alignment(mut self, alignment: u32) -> Self {
        self.bitmap_alignment = Some(alignment);
        self
    }

    /// Builds the [`Config`] struct using the settings configured in this builder.
    ///
    /// Returns [`None`] if failed to create [`Config`].
    pub fn build(self, lib: Arc<Library>) -> Option<Config> {
        let internal = unsafe { lib.ultralight().ulCreateConfig() };

        if internal.is_null() {
            return None;
        }

        set_config_str!(
            internal,
            self.cache_path,
            lib.ultralight().ulConfigSetCachePath
        );
        set_config_str!(
            internal,
            self.resource_path_prefix,
            lib.ultralight().ulConfigSetResourcePathPrefix
        );
        set_config!(
            internal,
            self.face_winding.map(|x| x as u32),
            lib.ultralight().ulConfigSetFaceWinding
        );
        set_config!(
            internal,
            self.font_hinting.map(|x| x as u32),
            lib.ultralight().ulConfigSetFontHinting
        );
        set_config!(
            internal,
            self.font_gamma,
            lib.ultralight().ulConfigSetFontGamma
        );
        set_config_str!(
            internal,
            self.user_stylesheet,
            lib.ultralight().ulConfigSetUserStylesheet
        );
        set_config!(
            internal,
            self.force_repaint,
            lib.ultralight().ulConfigSetForceRepaint
        );
        set_config!(
            internal,
            self.animation_timer_delay,
            lib.ultralight().ulConfigSetAnimationTimerDelay
        );
        set_config!(
            internal,
            self.scroll_timer_delay,
            lib.ultralight().ulConfigSetScrollTimerDelay
        );
        set_config!(
            internal,
            self.recycle_delay,
            lib.ultralight().ulConfigSetRecycleDelay
        );
        set_config!(
            internal,
            self.memory_cache_size,
            lib.ultralight().ulConfigSetMemoryCacheSize
        );
        set_config!(
            internal,
            self.page_cache_size,
            lib.ultralight().ulConfigSetPageCacheSize
        );
        set_config!(
            internal,
            self.override_ram_size,
            lib.ultralight().ulConfigSetOverrideRAMSize
        );
        set_config!(
            internal,
            self.min_large_heap_size,
            lib.ultralight().ulConfigSetMinLargeHeapSize
        );
        set_config!(
            internal,
            self.min_small_heap_size,
            lib.ultralight().ulConfigSetMinSmallHeapSize
        );
        set_config!(
            internal,
            self.num_renderer_threads,
            lib.ultralight().ulConfigSetNumRendererThreads
        );
        set_config!(
            internal,
            self.max_update_time,
            lib.ultralight().ulConfigSetMaxUpdateTime
        );
        set_config!(
            internal,
            self.bitmap_alignment,
            lib.ultralight().ulConfigSetBitmapAlignment
        );

        Some(Config { lib, internal })
    }
}
