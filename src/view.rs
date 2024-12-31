//! The View is a component used to load and display web content.
use std::{ops::Deref, sync::Arc};

use crate::{
    bitmap::BitmapFormat,
    error::CreationError,
    event::{KeyEvent, MouseEvent, ScrollEvent},
    javascript::JSContext,
    rect::Rect,
    renderer::Session,
    string::UlString,
    surface::Surface,
    Library,
};

/// Cursor types (See [`View::set_change_cursor_callback`] and [`Window::set_cursor`][crate::window::Window::set_cursor])
#[derive(Clone, Copy, Debug)]
pub enum Cursor {
    Alias = ul_sys::ULCursor_kCursor_Alias as isize,
    Cell = ul_sys::ULCursor_kCursor_Cell as isize,
    ColumnResize = ul_sys::ULCursor_kCursor_ColumnResize as isize,
    ContextMenu = ul_sys::ULCursor_kCursor_ContextMenu as isize,
    Copy = ul_sys::ULCursor_kCursor_Copy as isize,
    Cross = ul_sys::ULCursor_kCursor_Cross as isize,
    Custom = ul_sys::ULCursor_kCursor_Custom as isize,
    EastPanning = ul_sys::ULCursor_kCursor_EastPanning as isize,
    EastResize = ul_sys::ULCursor_kCursor_EastResize as isize,
    EastWestResize = ul_sys::ULCursor_kCursor_EastWestResize as isize,
    Grab = ul_sys::ULCursor_kCursor_Grab as isize,
    Grabbing = ul_sys::ULCursor_kCursor_Grabbing as isize,
    Hand = ul_sys::ULCursor_kCursor_Hand as isize,
    Help = ul_sys::ULCursor_kCursor_Help as isize,
    IBeam = ul_sys::ULCursor_kCursor_IBeam as isize,
    MiddlePanning = ul_sys::ULCursor_kCursor_MiddlePanning as isize,
    Move = ul_sys::ULCursor_kCursor_Move as isize,
    NoDrop = ul_sys::ULCursor_kCursor_NoDrop as isize,
    None = ul_sys::ULCursor_kCursor_None as isize,
    NorthEastPanning = ul_sys::ULCursor_kCursor_NorthEastPanning as isize,
    NorthEastResize = ul_sys::ULCursor_kCursor_NorthEastResize as isize,
    NorthEastSouthWestResize = ul_sys::ULCursor_kCursor_NorthEastSouthWestResize as isize,
    NorthPanning = ul_sys::ULCursor_kCursor_NorthPanning as isize,
    NorthResize = ul_sys::ULCursor_kCursor_NorthResize as isize,
    NorthSouthResize = ul_sys::ULCursor_kCursor_NorthSouthResize as isize,
    NorthWestPanning = ul_sys::ULCursor_kCursor_NorthWestPanning as isize,
    NorthWestResize = ul_sys::ULCursor_kCursor_NorthWestResize as isize,
    NorthWestSouthEastResize = ul_sys::ULCursor_kCursor_NorthWestSouthEastResize as isize,
    NotAllowed = ul_sys::ULCursor_kCursor_NotAllowed as isize,
    Pointer = ul_sys::ULCursor_kCursor_Pointer as isize,
    Progress = ul_sys::ULCursor_kCursor_Progress as isize,
    RowResize = ul_sys::ULCursor_kCursor_RowResize as isize,
    SouthEastPanning = ul_sys::ULCursor_kCursor_SouthEastPanning as isize,
    SouthEastResize = ul_sys::ULCursor_kCursor_SouthEastResize as isize,
    SouthPanning = ul_sys::ULCursor_kCursor_SouthPanning as isize,
    SouthResize = ul_sys::ULCursor_kCursor_SouthResize as isize,
    SouthWestPanning = ul_sys::ULCursor_kCursor_SouthWestPanning as isize,
    SouthWestResize = ul_sys::ULCursor_kCursor_SouthWestResize as isize,
    VerticalText = ul_sys::ULCursor_kCursor_VerticalText as isize,
    Wait = ul_sys::ULCursor_kCursor_Wait as isize,
    WestPanning = ul_sys::ULCursor_kCursor_WestPanning as isize,
    WestResize = ul_sys::ULCursor_kCursor_WestResize as isize,
    ZoomIn = ul_sys::ULCursor_kCursor_ZoomIn as isize,
    ZoomOut = ul_sys::ULCursor_kCursor_ZoomOut as isize,
}

impl TryFrom<ul_sys::ULCursor> for Cursor {
    type Error = ();

    fn try_from(value: ul_sys::ULCursor) -> Result<Self, Self::Error> {
        match value {
            ul_sys::ULCursor_kCursor_Alias => Ok(Self::Alias),
            ul_sys::ULCursor_kCursor_Cell => Ok(Self::Cell),
            ul_sys::ULCursor_kCursor_ColumnResize => Ok(Self::ColumnResize),
            ul_sys::ULCursor_kCursor_ContextMenu => Ok(Self::ContextMenu),
            ul_sys::ULCursor_kCursor_Copy => Ok(Self::Copy),
            ul_sys::ULCursor_kCursor_Cross => Ok(Self::Cross),
            ul_sys::ULCursor_kCursor_Custom => Ok(Self::Custom),
            ul_sys::ULCursor_kCursor_EastPanning => Ok(Self::EastPanning),
            ul_sys::ULCursor_kCursor_EastResize => Ok(Self::EastResize),
            ul_sys::ULCursor_kCursor_EastWestResize => Ok(Self::EastWestResize),
            ul_sys::ULCursor_kCursor_Grab => Ok(Self::Grab),
            ul_sys::ULCursor_kCursor_Grabbing => Ok(Self::Grabbing),
            ul_sys::ULCursor_kCursor_Hand => Ok(Self::Hand),
            ul_sys::ULCursor_kCursor_Help => Ok(Self::Help),
            ul_sys::ULCursor_kCursor_IBeam => Ok(Self::IBeam),
            ul_sys::ULCursor_kCursor_MiddlePanning => Ok(Self::MiddlePanning),
            ul_sys::ULCursor_kCursor_Move => Ok(Self::Move),
            ul_sys::ULCursor_kCursor_NoDrop => Ok(Self::NoDrop),
            ul_sys::ULCursor_kCursor_None => Ok(Self::None),
            ul_sys::ULCursor_kCursor_NorthEastPanning => Ok(Self::NorthEastPanning),
            ul_sys::ULCursor_kCursor_NorthEastResize => Ok(Self::NorthEastResize),
            ul_sys::ULCursor_kCursor_NorthEastSouthWestResize => Ok(Self::NorthEastSouthWestResize),
            ul_sys::ULCursor_kCursor_NorthPanning => Ok(Self::NorthPanning),
            ul_sys::ULCursor_kCursor_NorthResize => Ok(Self::NorthResize),
            ul_sys::ULCursor_kCursor_NorthSouthResize => Ok(Self::NorthSouthResize),
            ul_sys::ULCursor_kCursor_NorthWestPanning => Ok(Self::NorthWestPanning),
            ul_sys::ULCursor_kCursor_NorthWestResize => Ok(Self::NorthWestResize),
            ul_sys::ULCursor_kCursor_NorthWestSouthEastResize => Ok(Self::NorthWestSouthEastResize),
            ul_sys::ULCursor_kCursor_NotAllowed => Ok(Self::NotAllowed),
            ul_sys::ULCursor_kCursor_Pointer => Ok(Self::Pointer),
            ul_sys::ULCursor_kCursor_Progress => Ok(Self::Progress),
            ul_sys::ULCursor_kCursor_RowResize => Ok(Self::RowResize),
            ul_sys::ULCursor_kCursor_SouthEastPanning => Ok(Self::SouthEastPanning),
            ul_sys::ULCursor_kCursor_SouthEastResize => Ok(Self::SouthEastResize),
            ul_sys::ULCursor_kCursor_SouthPanning => Ok(Self::SouthPanning),
            ul_sys::ULCursor_kCursor_SouthResize => Ok(Self::SouthResize),
            ul_sys::ULCursor_kCursor_SouthWestPanning => Ok(Self::SouthWestPanning),
            ul_sys::ULCursor_kCursor_SouthWestResize => Ok(Self::SouthWestResize),
            ul_sys::ULCursor_kCursor_VerticalText => Ok(Self::VerticalText),
            ul_sys::ULCursor_kCursor_Wait => Ok(Self::Wait),
            ul_sys::ULCursor_kCursor_WestPanning => Ok(Self::WestPanning),
            ul_sys::ULCursor_kCursor_WestResize => Ok(Self::WestResize),
            ul_sys::ULCursor_kCursor_ZoomIn => Ok(Self::ZoomIn),
            ul_sys::ULCursor_kCursor_ZoomOut => Ok(Self::ZoomOut),
            _ => Err(()),
        }
    }
}

/// Rendering details for a View, to be used with your own GPUDriver
///
/// When using your own [`GpuDriver`](crate::gpu_driver::GpuDriver), each
/// [`View`] is rendered to an offscreen texture that you can
/// display on a 3D quad in your application. This struct provides all the
/// details you need to display the corresponding texture in your application.
#[derive(Clone, Copy, Debug)]
pub struct RenderTarget {
    /// Whether this target is empty (null texture)
    pub is_empty: bool,
    /// The viewport width (in device coordinates).
    pub width: u32,
    /// The viewport height (in device coordinates).
    pub height: u32,
    /// The GPUDriver-specific texture ID (the texture will be created
    /// in [`GpuDriver::create_texture`](crate::gpu_driver::GpuDriver::create_texture)).
    pub texture_id: u32,
    /// The texture width (in pixels). This may be padded.
    pub texture_width: u32,
    /// The texture height (in pixels). This may be padded.
    pub texture_height: u32,
    /// The pixel format of the texture.
    pub texture_format: BitmapFormat,
    /// UV coordinates of the texture (this is needed because the texture may be padded).
    pub uv_coords: Rect<f32>,
    /// The GPUDriver-specific render buffer ID (the render buffer will be created
    /// in [`GpuDriver::create_render_buffer`](crate::gpu_driver::GpuDriver::create_render_buffer)).
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

/// Console message source types (See [`View::set_add_console_message_callback`])
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum ConsoleMessageSource {
    XML = ul_sys::ULMessageSource_kMessageSource_XML as isize,
    JS = ul_sys::ULMessageSource_kMessageSource_JS as isize,
    Network = ul_sys::ULMessageSource_kMessageSource_Network as isize,
    ConsoleAPI = ul_sys::ULMessageSource_kMessageSource_ConsoleAPI as isize,
    Storage = ul_sys::ULMessageSource_kMessageSource_Storage as isize,
    AppCache = ul_sys::ULMessageSource_kMessageSource_AppCache as isize,
    Rendering = ul_sys::ULMessageSource_kMessageSource_Rendering as isize,
    CSS = ul_sys::ULMessageSource_kMessageSource_CSS as isize,
    Security = ul_sys::ULMessageSource_kMessageSource_Security as isize,
    ContentBlocker = ul_sys::ULMessageSource_kMessageSource_ContentBlocker as isize,
    Media = ul_sys::ULMessageSource_kMessageSource_Media as isize,
    MediaSource = ul_sys::ULMessageSource_kMessageSource_MediaSource as isize,
    WebRTC = ul_sys::ULMessageSource_kMessageSource_WebRTC as isize,
    ITPDebug = ul_sys::ULMessageSource_kMessageSource_ITPDebug as isize,
    PrivateClickMeasurement =
        ul_sys::ULMessageSource_kMessageSource_PrivateClickMeasurement as isize,
    PaymentRequest = ul_sys::ULMessageSource_kMessageSource_PaymentRequest as isize,
    Other = ul_sys::ULMessageSource_kMessageSource_Other as isize,
}

impl TryFrom<ul_sys::ULMessageSource> for ConsoleMessageSource {
    type Error = ();
    fn try_from(value: ul_sys::ULMessageSource) -> Result<Self, Self::Error> {
        match value {
            ul_sys::ULMessageSource_kMessageSource_XML => Ok(ConsoleMessageSource::XML),
            ul_sys::ULMessageSource_kMessageSource_JS => Ok(ConsoleMessageSource::JS),
            ul_sys::ULMessageSource_kMessageSource_Network => Ok(ConsoleMessageSource::Network),
            ul_sys::ULMessageSource_kMessageSource_ConsoleAPI => {
                Ok(ConsoleMessageSource::ConsoleAPI)
            }
            ul_sys::ULMessageSource_kMessageSource_Storage => Ok(ConsoleMessageSource::Storage),
            ul_sys::ULMessageSource_kMessageSource_AppCache => Ok(ConsoleMessageSource::AppCache),
            ul_sys::ULMessageSource_kMessageSource_Rendering => Ok(ConsoleMessageSource::Rendering),
            ul_sys::ULMessageSource_kMessageSource_CSS => Ok(ConsoleMessageSource::CSS),
            ul_sys::ULMessageSource_kMessageSource_Security => Ok(ConsoleMessageSource::Security),
            ul_sys::ULMessageSource_kMessageSource_ContentBlocker => {
                Ok(ConsoleMessageSource::ContentBlocker)
            }
            ul_sys::ULMessageSource_kMessageSource_Other => Ok(ConsoleMessageSource::Other),
            _ => Err(()),
        }
    }
}

/// Console message levels (See [`View::set_add_console_message_callback`])
#[derive(Clone, Copy, Debug)]
pub enum ConsoleMessageLevel {
    Log = ul_sys::ULMessageLevel_kMessageLevel_Log as isize,
    Warning = ul_sys::ULMessageLevel_kMessageLevel_Warning as isize,
    Error = ul_sys::ULMessageLevel_kMessageLevel_Error as isize,
    Debug = ul_sys::ULMessageLevel_kMessageLevel_Debug as isize,
    Info = ul_sys::ULMessageLevel_kMessageLevel_Info as isize,
}

impl TryFrom<ul_sys::ULMessageLevel> for ConsoleMessageLevel {
    type Error = ();
    fn try_from(value: ul_sys::ULMessageLevel) -> Result<Self, ()> {
        match value {
            ul_sys::ULMessageLevel_kMessageLevel_Log => Ok(ConsoleMessageLevel::Log),
            ul_sys::ULMessageLevel_kMessageLevel_Warning => Ok(ConsoleMessageLevel::Warning),
            ul_sys::ULMessageLevel_kMessageLevel_Error => Ok(ConsoleMessageLevel::Error),
            ul_sys::ULMessageLevel_kMessageLevel_Debug => Ok(ConsoleMessageLevel::Debug),
            ul_sys::ULMessageLevel_kMessageLevel_Info => Ok(ConsoleMessageLevel::Info),
            _ => Err(()),
        }
    }
}

/// Configuration to be used when creating a [`View`].
pub struct ViewConfig {
    lib: Arc<Library>,
    internal: ul_sys::ULViewConfig,
}

impl ViewConfig {
    /// Starts the building process for the [`ViewConfig`] struct. returns a builder
    /// which can be used to configure the settings.
    pub fn start() -> ViewConfigBuilder {
        ViewConfigBuilder::default()
    }

    /// Returns the underlying [`ul_sys::ULViewConfig`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULViewConfig {
        self.internal
    }
}

impl Drop for ViewConfig {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().ulDestroyViewConfig(self.internal);
        }
    }
}

/// Builder for the [`ViewConfig`] struct.
#[derive(Default)]
pub struct ViewConfigBuilder {
    is_accelerated: Option<bool>,
    is_transparent: Option<bool>,
    initial_device_scale: Option<f64>,
    initial_focus: Option<bool>,
    enable_images: Option<bool>,
    enable_javascript: Option<bool>,
    font_family_standard: Option<String>,
    font_family_fixed: Option<String>,
    font_family_serif: Option<String>,
    font_family_sans_serif: Option<String>,
    user_agent: Option<String>,
    // display_id: Option<u32>,
}

impl ViewConfigBuilder {
    /// Whether to render using the GPU renderer (accelerated) or the CPU renderer (unaccelerated).
    ///
    /// When `true`, the View will be rendered to an offscreen GPU texture using the GPU driver set in
    /// [`platform::set_gpu_driver`](crate::platform::set_gpu_driver).
    /// You can fetch details for the texture via [`View::render_target`].
    ///
    /// When `false` (the default), the View will be rendered to an offscreen
    /// pixel buffer using the multithreaded CPU renderer.
    pub fn is_accelerated(mut self, is_accelerated: bool) -> Self {
        self.is_accelerated = Some(is_accelerated);
        self
    }

    /// Whether or not this View should support transparency.
    ///
    /// Make sure to also set the following CSS on the page:
    /// ```css
    /// html, body { background: transparent; }
    /// ```
    ///
    /// default is `false`
    pub fn is_transparent(mut self, is_transparent: bool) -> Self {
        self.is_transparent = Some(is_transparent);
        self
    }

    /// The initial device scale, ie. the amount to scale page units to screen
    /// pixels. This should be set to the scaling factor of the device that
    /// the View is displayed on.
    ///
    /// 1.0 is equal to 100% zoom (no scaling), 2.0 is equal to 200% zoom (2x scaling)
    ///
    /// default is `1.0`
    pub fn initial_device_scale(mut self, initial_device_scale: f64) -> Self {
        self.initial_device_scale = Some(initial_device_scale);
        self
    }

    /// Whether or not the View should initially have input focus, [`View::focus`].
    ///
    /// default is `false`
    pub fn initial_focus(mut self, initial_focus: bool) -> Self {
        self.initial_focus = Some(initial_focus);
        self
    }

    /// Whether or not images should be enabled.
    ///
    /// default is `true`
    pub fn enable_images(mut self, enable_images: bool) -> Self {
        self.enable_images = Some(enable_images);
        self
    }

    /// Whether or not JavaScript should be enabled.
    ///
    /// default is `true`
    pub fn enable_javascript(mut self, enable_javascript: bool) -> Self {
        self.enable_javascript = Some(enable_javascript);
        self
    }

    /// Default font-family to use.
    ///
    /// default is `"Times New Roman"`
    pub fn font_family_standard(mut self, font_family_standard: &str) -> Self {
        self.font_family_standard = Some(font_family_standard.to_string());
        self
    }

    /// Default font-family to use for fixed fonts. (pre/code)
    ///
    /// default is `"Courier New"`
    pub fn font_family_fixed(mut self, font_family_fixed: &str) -> Self {
        self.font_family_fixed = Some(font_family_fixed.to_string());
        self
    }

    /// Default font-family to use for serif fonts.
    ///
    /// default is `"Times New Roman"`
    pub fn font_family_serif(mut self, font_family_serif: &str) -> Self {
        self.font_family_serif = Some(font_family_serif.to_string());
        self
    }

    /// Default font-family to use for sans-serif fonts.
    ///
    /// default is `"Arial"`
    pub fn font_family_sans_serif(mut self, font_family_sans_serif: &str) -> Self {
        self.font_family_sans_serif = Some(font_family_sans_serif.to_string());
        self
    }

    /// User-agent string to use.
    ///
    /// default is
    /// `"Mozilla/5.0 (Windows NT 10.0; Win64; x64)
    ///   AppleWebKit/608.3.10 (KHTML, like Gecko)
    ///   Ultralight/1.3.0 Safari/608.3.10"`
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    // TODO: ulViewConfigSetDisplayId isn't found in the library we use from github, but is found
    //       in the one from the website. fix that.
    // /// A user-generated id for the display (monitor, TV, or screen) that this View will be shown on.
    // ///
    // /// Animations are driven based on the physical refresh rate of the display. Multiple Views can
    // /// share the same display.
    // ///
    // /// Note: This is automatically managed for you when [`App`][crate::app::App] is used.
    // ///
    // /// See also [`Renderer::refresh_display`][crate::renderer::Renderer::refresh_display].
    // pub fn display_id(mut self, display_id: u32) -> Self {
    //     self.display_id = Some(display_id);
    //     self
    // }

    /// Builds the [`ViewConfig`] struct using the settings configured in this builder.
    ///
    /// Returns [`None`] if failed to create [`ViewConfig`].
    pub fn build(self, lib: Arc<Library>) -> Option<ViewConfig> {
        let internal = unsafe { lib.ultralight().ulCreateViewConfig() };

        if internal.is_null() {
            return None;
        }

        set_config!(
            internal,
            self.is_accelerated,
            lib.ultralight().ulViewConfigSetIsAccelerated
        );
        set_config!(
            internal,
            self.is_transparent,
            lib.ultralight().ulViewConfigSetIsTransparent
        );
        set_config!(
            internal,
            self.initial_device_scale,
            lib.ultralight().ulViewConfigSetInitialDeviceScale
        );
        set_config!(
            internal,
            self.initial_focus,
            lib.ultralight().ulViewConfigSetInitialFocus
        );
        set_config!(
            internal,
            self.enable_images,
            lib.ultralight().ulViewConfigSetEnableImages
        );
        set_config!(
            internal,
            self.enable_javascript,
            lib.ultralight().ulViewConfigSetEnableJavaScript
        );
        set_config_str!(
            internal,
            self.font_family_standard,
            lib.ultralight().ulViewConfigSetFontFamilyStandard
        );
        set_config_str!(
            internal,
            self.font_family_fixed,
            lib.ultralight().ulViewConfigSetFontFamilyFixed
        );
        set_config_str!(
            internal,
            self.font_family_serif,
            lib.ultralight().ulViewConfigSetFontFamilySerif
        );
        set_config_str!(
            internal,
            self.font_family_sans_serif,
            lib.ultralight().ulViewConfigSetFontFamilySansSerif
        );
        set_config_str!(
            internal,
            self.user_agent,
            lib.ultralight().ulViewConfigSetUserAgent
        );
        // set_config!(internal, self.display_id, ulViewConfigSetDisplayId);

        Some(ViewConfig { lib, internal })
    }
}

/// An RAII implementation of a “scoped lock” of a pixel buffer for Javascript Context
/// of [`View`].
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
///
/// This struct is created by [`View::lock_js_context`].
///
/// This can be used as [`Deref`] to access the underlying [`JSContext`].
pub struct ViewJSContextGuard<'a> {
    view: &'a View,
    js_ctx: JSContext,
}

impl Deref for ViewJSContextGuard<'_> {
    type Target = JSContext;

    fn deref(&self) -> &Self::Target {
        &self.js_ctx
    }
}

impl Drop for ViewJSContextGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.view
                .lib
                .ultralight()
                .ulViewUnlockJSContext(self.view.internal);
        }
    }
}

/// The View class is used to load and display web content.
///
/// View is an offscreen web-page container that can be used to display web-content in your
/// application.
///
/// You can load content into a View via [`View::load_url`] or [`View::load_html`]
/// and interact with it via [`View::fire_mouse_event`] and similar API.
///
/// When displaying a View, the API is different depending on whether you are
/// using the CPU renderer or the GPU renderer:
///
/// When using the CPU renderer, you would get the underlying pixel-buffer
/// surface for a View via [`View::surface`].
///
/// When using the GPU renderer, you would get the underlying render target
/// and texture information via [`View::render_target`].
pub struct View {
    lib: Arc<Library>,
    internal: ul_sys::ULView,
    need_to_destroy: bool,
}

impl View {
    /// Helper internal function to allow getting a reference to a managed
    /// session.
    pub(crate) unsafe fn from_raw(lib: Arc<Library>, raw: ul_sys::ULView) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self {
                lib,
                internal: raw,
                need_to_destroy: false,
            })
        }
    }

    /// Internal function helper to create a view.
    /// (See [`Renderer::create_view`](crate::renderer::Renderer::create_view))
    pub(crate) unsafe fn create(
        renderer: ul_sys::ULRenderer,
        width: u32,
        height: u32,
        view_config: &ViewConfig,
        session: Option<&Session>,
    ) -> Option<Self> {
        let lib = view_config.lib.clone();
        let internal = lib.ultralight().ulCreateView(
            renderer,
            width,
            height,
            view_config.to_ul(),
            session.map(|s| s.to_ul()).unwrap_or(std::ptr::null_mut()),
        );

        if internal.is_null() {
            None
        } else {
            Some(Self {
                lib,
                internal,
                need_to_destroy: true,
            })
        }
    }

    /// Returns the underlying [`ul_sys::ULView`] struct, to be used locally for
    /// calling the underlying C API.
    #[allow(dead_code)]
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULView {
        self.internal
    }
}

impl View {
    /// Get the URL of the current page loaded into this View, if any.
    pub fn url(&self) -> Result<String, CreationError> {
        unsafe {
            let url_string = self.lib.ultralight().ulViewGetURL(self.internal);
            UlString::copy_raw_to_string(&self.lib, url_string)
        }
    }

    /// Get the title of the current page loaded into this View, if any.
    pub fn title(&self) -> Result<String, CreationError> {
        unsafe {
            let title_string = self.lib.ultralight().ulViewGetTitle(self.internal);
            UlString::copy_raw_to_string(&self.lib, title_string)
        }
    }

    /// Get the width of the View, in pixels.
    pub fn width(&self) -> u32 {
        unsafe { self.lib.ultralight().ulViewGetWidth(self.internal) }
    }

    /// Get the height of the View, in pixels.
    pub fn height(&self) -> u32 {
        unsafe { self.lib.ultralight().ulViewGetHeight(self.internal) }
    }

    /// Get the device scale, ie. the amount to scale page units to screen pixels.
    ///
    /// For example, a value of 1.0 is equivalent to 100% zoom. A value of 2.0 is 200% zoom.
    pub fn device_scale(&self) -> f64 {
        unsafe { self.lib.ultralight().ulViewGetDeviceScale(self.internal) }
    }

    /// Set the device scale.
    pub fn set_device_scale(&self, scale: f64) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewSetDeviceScale(self.internal, scale)
        }
    }

    /// Whether or not the View is GPU-accelerated. If this is false,
    /// the page will be rendered via the CPU renderer.
    pub fn is_accelerated(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewIsAccelerated(self.internal) }
    }

    /// Whether or not the View supports transparent backgrounds.
    pub fn is_transparent(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewIsTransparent(self.internal) }
    }

    /// Check if the main frame of the page is currently loading.
    pub fn is_loading(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewIsLoading(self.internal) }
    }

    /// Get the RenderTarget for the View.
    ///
    /// Only valid when the view is accelerated, and will return [`None`] otherwise.
    pub fn render_target(&self) -> Option<RenderTarget> {
        if self.is_accelerated() {
            Some(unsafe {
                RenderTarget::from(self.lib.ultralight().ulViewGetRenderTarget(self.internal))
            })
        } else {
            None
        }
    }

    /// Get the Surface for the View (native pixel buffer container).
    ///
    /// Only valid when the view is not accelerated, and will return [`None`] otherwise.
    pub fn surface(&self) -> Option<Surface> {
        if !self.is_accelerated() {
            unsafe {
                let surface = self.lib.ultralight().ulViewGetSurface(self.internal);
                if surface.is_null() {
                    None
                } else {
                    Some(Surface::from_raw(self.lib.clone(), surface))
                }
            }
        } else {
            None
        }
    }

    /// Load a raw string of HTML, the View will navigate to it as a new page.
    pub fn load_html(&self, html: &str) -> Result<(), CreationError> {
        unsafe {
            let ul_string = UlString::from_str(self.lib.clone(), html)?;
            self.lib
                .ultralight()
                .ulViewLoadHTML(self.internal, ul_string.to_ul());
        }
        Ok(())
    }

    /// Load a URL, the View will navigate to it as a new page.
    ///
    /// You can use File URLs (eg, file:///page.html) as well.
    pub fn load_url(&self, url: &str) -> Result<(), CreationError> {
        unsafe {
            let ul_string = UlString::from_str(self.lib.clone(), url)?;
            self.lib
                .ultralight()
                .ulViewLoadURL(self.internal, ul_string.to_ul());
        }
        Ok(())
    }

    /// Resize View to a certain size.
    ///
    /// # Arguments
    /// * `width` - The new width in pixels.
    /// * `height` - The new height in pixels.
    pub fn resize(&self, width: u32, height: u32) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewResize(self.internal, width, height);
        }
    }

    /// Acquire the page's [`JSContext`] for use with JavaScriptCore API.
    ///
    /// Note: This call locks the context for the current thread. You should call
    /// You should drop this so that the context is unlocked so that other threads
    /// can use this context.
    ///
    /// The lock is recusive, it's okay to aquaire this multiple times, but you should drop
    /// all the instances to unlock the context.
    pub fn lock_js_context(&self) -> ViewJSContextGuard {
        let ctx = unsafe { self.lib.ultralight().ulViewLockJSContext(self.internal) };

        let js_ctx = JSContext::copy_from_raw(self.lib.clone(), ctx);

        ViewJSContextGuard { view: self, js_ctx }
    }

    /// Helper function to evaluate a raw string of JavaScript and return the result as a String.
    ///
    /// You can pass the raw Javascript string in `script`, if an exception occurs
    /// it will be returned in [`Err`], otherwise a string result in [`Ok`] will be
    /// returned.
    pub fn evaluate_script(&self, script: &str) -> Result<Result<String, String>, CreationError> {
        unsafe {
            let ul_script_string = UlString::from_str(self.lib.clone(), script)?;
            // a dummy value, it will be replaced by the actual result
            let mut exception_string = 1 as ul_sys::ULString;
            let result_string = self.lib.ultralight().ulViewEvaluateScript(
                self.internal,
                ul_script_string.to_ul(),
                &mut exception_string as _,
            );

            let has_exception = !self.lib.ultralight().ulStringIsEmpty(exception_string);
            if has_exception {
                let exception_string = UlString::copy_raw_to_string(&self.lib, exception_string)?;
                Ok(Err(exception_string))
            } else {
                let result_string = UlString::copy_raw_to_string(&self.lib, result_string)?;
                Ok(Ok(result_string))
            }
        }
    }

    /// Whether or not we can navigate backwards in history
    pub fn can_go_back(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewCanGoBack(self.internal) }
    }

    /// Whether or not we can navigate forwards in history
    pub fn can_go_forward(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewCanGoForward(self.internal) }
    }

    /// Navigate backwards in history
    pub fn go_back(&self) {
        unsafe { self.lib.ultralight().ulViewGoBack(self.internal) }
    }

    /// Navigate forwards in history
    pub fn go_forward(&self) {
        unsafe { self.lib.ultralight().ulViewGoForward(self.internal) }
    }

    /// Navigate to an arbitrary offset in history
    pub fn go_to_history_offset(&self, offset: i32) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewGoToHistoryOffset(self.internal, offset)
        }
    }

    /// Reload current page
    pub fn reload(&self) {
        unsafe { self.lib.ultralight().ulViewReload(self.internal) }
    }

    /// Stop all page loads
    pub fn stop(&self) {
        unsafe { self.lib.ultralight().ulViewStop(self.internal) }
    }

    /// Give focus to the View.
    ///
    /// You should call this to give visual indication that the View
    /// has input focus (changes active text selection colors, for example).
    pub fn focus(&self) {
        unsafe { self.lib.ultralight().ulViewFocus(self.internal) }
    }

    /// Remove focus from the View and unfocus any focused input elements.
    ///
    /// You should call this to give visual indication that the View has lost input focus.
    pub fn unfocus(&self) {
        unsafe { self.lib.ultralight().ulViewUnfocus(self.internal) }
    }

    /// Whether or not the View has focus.
    pub fn has_focus(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewHasFocus(self.internal) }
    }

    /// Whether or not the View has an input element with visible keyboard focus
    /// (indicated by a blinking caret).
    ///
    /// You can use this to decide whether or not the View should consume
    /// keyboard input events (useful in games with mixed UI and key handling).
    pub fn has_input_focus(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewHasInputFocus(self.internal) }
    }

    /// Fire a keyboard event
    ///
    /// Note that only [`KeyEventType::Char`](crate::event::KeyEventType::Char)
    /// events actually generate text in input fields.
    pub fn fire_key_event(&self, key_event: KeyEvent) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewFireKeyEvent(self.internal, key_event.to_ul())
        }
    }

    /// Fire a mouse event
    pub fn fire_mouse_event(&self, mouse_event: MouseEvent) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewFireMouseEvent(self.internal, mouse_event.to_ul())
        }
    }

    /// Fire a scroll event
    pub fn fire_scroll_event(&self, scroll_event: ScrollEvent) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewFireScrollEvent(self.internal, scroll_event.to_ul())
        }
    }

    /// Get the display id of the View.
    pub fn get_display_id(&self) -> u32 {
        unsafe { self.lib.ultralight().ulViewGetDisplayId(self.internal) }
    }

    /// Set the display id of the View.
    ///
    /// This should be called when the View is moved to another display.
    pub fn set_display_id(&self, display_id: u32) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewSetDisplayId(self.internal, display_id)
        }
    }

    // looking at the CPP header, the strings seems to be references
    // but the C headers doesn't say we must not destroy them.
    // For now we don't destroy.
    //  TODO: check if we don't need to destroy them
    set_callback! {
        /// Called when the page title changes
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `title: String` - The new title
        pub fn set_change_title_callback(&self, callback: FnMut(view: &View, title: String)) :
            [View::lib.ultralight()][s] ulViewSetChangeTitleCallback(ul_view: ul_sys::ULView, ul_title: ul_sys::ULString) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let title = UlString::copy_raw_to_string(&s.lib,ul_title).unwrap();
        }
    }

    set_callback! {
        /// Called when the page URL changes
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `url: String` - The new url
        pub fn set_change_url_callback(&self, callback: FnMut(view: &View, url: String)) :
            [View::lib.ultralight()][s] ulViewSetChangeURLCallback(ul_view: ul_sys::ULView, ul_url: ul_sys::ULString) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib,ul_url).unwrap();
        }
    }

    set_callback! {
        /// Called when the tooltip changes (usually as result of a mouse hover)
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `tooltip: String` - The tooltip string
        pub fn set_change_tooltip_callback(&self, callback: FnMut(view: &View, tooltip: String)) :
            [View::lib.ultralight()][s] ulViewSetChangeTooltipCallback(ul_view: ul_sys::ULView, ul_tooltip: ul_sys::ULString) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let tooltip = UlString::copy_raw_to_string(&s.lib,ul_tooltip).unwrap();
        }
    }

    set_callback! {
        /// Called when the mouse cursor changes
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `cursor: Cursor` - The cursor type
        pub fn set_change_cursor_callback(&self, callback: FnMut(view: &View, cursor: Cursor)) :
            [View::lib.ultralight()][s] ulViewSetChangeCursorCallback(ul_view: ul_sys::ULView, ul_cursor: ul_sys::ULCursor) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let cursor = Cursor::try_from(ul_cursor).unwrap();
        }
    }

    set_callback! {
        /// Called when a message is added to the console (useful for errors / debug)
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `message_source: ConsoleMessageSource` - The source of the message
        /// * `message_level: ConsoleMessageLevel` - The level of the message
        /// * `message: String` - The message
        /// * `line_number: i32` - The line number of the message
        /// * `column_number: i32` - The column number of the message
        /// * `source_id: String` - The source id of the message
        pub fn set_add_console_message_callback(&self, callback: FnMut(
                view: &View,
                message_source: ConsoleMessageSource,
                message_level: ConsoleMessageLevel,
                message: String,
                line_number:u32,
                column_number:u32,
                source_id: String)) :
            [View::lib.ultralight()][s] ulViewSetAddConsoleMessageCallback(
               ul_view: ul_sys::ULView,
               ul_message_source: ul_sys::ULMessageSource,
               ul_message_level: ul_sys::ULMessageLevel,
               ul_message: ul_sys::ULString,
               line_number: u32,
               column_number :u32,
               ul_source_id: ul_sys::ULString
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let message_source = ConsoleMessageSource::try_from(ul_message_source).unwrap();
               let message_level = ConsoleMessageLevel::try_from(ul_message_level).unwrap();
               let message = UlString::copy_raw_to_string(&s.lib,ul_message).unwrap();
               let source_id = UlString::copy_raw_to_string(&s.lib,ul_source_id).unwrap();
        }
    }

    set_callback! {
        // TODO: this callback require that you return owned `View`
        //       but because you have to render it yourself, this won't do,
        //       its better to return a reference, but not sure how we should
        //       manage the owner and lifetime.
        //       You can return `None` and create a new view since you have
        //       the `url` and all information needed to create it.
        //
        /// Set callback for when the page wants to create a new View.
        ///
        /// This is usually the result of a user clicking a link with
        /// `target="_blank"` or by JavaScript calling `window.open(url)`.
        ///
        /// To allow creation of these new Views, you should create a new View
        /// in this callback (eg. [`Renderer::create_view`](crate::renderer::Renderer::create_view)),
        /// resize it to your container, and return it.
        /// You are responsible for displaying the returned View.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `opener_url: String` - The url of the page that initiated this request
        /// * `target_url: String` - The url the new View will navigate to
        /// * `is_popup: bool` - Whether or not this was triggered by window.open()
        /// * `popup_rect: Rect<i32>` - Popups can optionally request certain
        /// dimensions and coordinates via window.open(). You can choose to
        /// respect these or not by resizing/moving the View to this rect.
        ///
        /// You should return [`None`] if you want to block the action.
        pub fn set_create_child_view_callback(&self, callback: FnMut(
                view: &View,
                opener_url: String,
                target_url: String,
                is_popup: bool,
                popup_rect: Rect<i32>
                // TODO: should we change the return type?
                //       since the new view will be owned by another overlay
            ) -> ret_view: Option<View>) :
            [View::lib.ultralight()][s] ulViewSetCreateChildViewCallback(
               ul_view: ul_sys::ULView,
               ul_opener_url: ul_sys::ULString,
               ul_target_url: ul_sys::ULString,
               is_popup: bool,
               ul_popup_rect: ul_sys::ULIntRect
            ) -> ul_sys::ULView {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let opener_url = UlString::copy_raw_to_string(&s.lib,ul_opener_url).unwrap();
               let target_url = UlString::copy_raw_to_string(&s.lib,ul_target_url).unwrap();
               let popup_rect = Rect::from(ul_popup_rect);
        } {
            if let Some(ret_view) = ret_view {
                ret_view.internal
            } else {
                std::ptr::null_mut()
            }
        }
    }

    set_callback! {
        // TODO: this callback require that you return owned `View`
        //       but because you have to render it yourself, this won't do,
        //       its better to return a reference, but not sure how we should
        //       manage the owner and lifetime.
        //       You can return `None` and create a new view since you have
        //       the `url` and all information needed to create it.
        //
        /// Set callback for when the page wants to create a new View to display the
        /// local inspector in.
        ///
        /// See also [`View::create_local_inspector_view`].
        ///
        /// To allow creation of these new Views, you should create a new View
        /// in this callback (eg. [`Renderer::create_view`](crate::renderer::Renderer::create_view)),
        /// resize it to your container, and return it.
        /// You are responsible for displaying the returned View.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `is_local: bool` - Whether or not this inspector view is local
        /// * `inspected_url: String` - The url of the page that initiated this request
        ///
        /// You should return [`None`] if you want to block the action.
        pub fn set_create_inspector_view_callback(&self, callback: FnMut(
                view: &View,
                is_local: bool,
                inspected_url: String
                // TODO: should we change the return type?
                //       since the new view will be owned by another overlay
            ) -> ret_view: Option<View>) :
            [View::lib.ultralight()][s] ulViewSetCreateInspectorViewCallback(
               ul_view: ul_sys::ULView,
               is_local: bool,
               ul_inspected_url: ul_sys::ULString
            ) -> ul_sys::ULView {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
                let inspected_url = UlString::copy_raw_to_string(&s.lib,ul_inspected_url).unwrap();
        } {
            if let Some(ret_view) = ret_view {
                ret_view.internal
            } else {
                std::ptr::null_mut()
            }
        }
    }

    set_callback! {
        /// Called when the page begins loading a new URL into a frame.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `frame_id: u64` - A unique ID for the frame
        /// * `is_main_frame: bool` - Whether or not this is the main frame
        /// * `url: String` - The url that is being loaded
        pub fn set_begin_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
            [View::lib.ultralight()][s] ulViewSetBeginLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib,ul_url).unwrap();
        }
    }

    set_callback! {
        /// Called when the page finishes loading a URL into a frame.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `frame_id: u64` - A unique ID for the frame
        /// * `is_main_frame: bool` - Whether or not this is the main frame
        /// * `url: String` - The url that is being loaded
        pub fn set_finish_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
            [View::lib.ultralight()][s] ulViewSetFinishLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib,ul_url).unwrap();
        }
    }

    set_callback! {
        /// Called when an error occurs while loading a URL into a frame.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `frame_id: u64` - A unique ID for the frame
        /// * `is_main_frame: bool` - Whether or not this is the main frame
        /// * `url: String` - The url that is being loaded
        /// * `description: String` -  A human-readable description of the error.
        /// * `error_domain: String` - The name of the module that triggered the error.
        /// * `error_code: u32` - Internal error code generated by the module
        pub fn set_fail_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String,
                description: String,
                error_domain: String,
                error_code: i32)) :
            [View::lib.ultralight()][s] ulViewSetFailLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString,
               ul_description: ul_sys::ULString,
               ul_error_domain: ul_sys::ULString,
               error_code: i32
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib,ul_url).unwrap();
               let description = UlString::copy_raw_to_string(&s.lib,ul_description).unwrap();
               let error_domain = UlString::copy_raw_to_string(&s.lib,ul_error_domain).unwrap();
        }
    }

    set_callback! {
        /// Set callback for when the JavaScript window object is reset for a new page load.
        ///
        /// This is called before any scripts are executed on the page and is the earliest time to setup any
        /// initial JavaScript state or bindings.
        ///
        /// The document is not guaranteed to be loaded/parsed at this point. If you need to make any
        /// JavaScript calls that are dependent on DOM elements or scripts on the page, use DOMReady
        /// instead.
        ///
        /// The window object is lazily initialized (this will not be called on pages with no scripts).
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `frame_id: u64` - A unique ID for the frame
        /// * `is_main_frame: bool` - Whether or not this is the main frame
        /// * `url: String` - The url that is being loaded
        pub fn set_window_object_ready_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
            [View::lib.ultralight()][s] ulViewSetWindowObjectReadyCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib,ul_url).unwrap();
        }
    }

    set_callback! {
        /// Called when all JavaScript has been parsed and the document is ready.
        ///
        /// This is the best time to make any JavaScript calls that are dependent
        /// on DOM elements or scripts on the page.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        /// * `frame_id: u64` - A unique ID for the frame
        /// * `is_main_frame: bool` - Whether or not this is the main frame
        /// * `url: String` - The url that is being loaded
        pub fn set_dom_ready_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
            [View::lib.ultralight()][s] ulViewSetDOMReadyCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
               let url = UlString::copy_raw_to_string(&s.lib, ul_url).unwrap();
        }
    }

    set_callback! {
        /// Called when the session history (back/forward state) is modified.
        ///
        /// # Callback Arguments
        /// * `view: &View` - The view that fired the event (eg. self)
        pub fn set_update_history_callback(&self, callback: FnMut(view: &View)) :
           [View::lib.ultralight()][s] ulViewSetUpdateHistoryCallback(ul_view: ul_sys::ULView) {
               let view = &View::from_raw(s.lib.clone(), ul_view).unwrap();
        }
    }

    /// Set whether or not this View should be repainted during the next call
    /// to [`Renderer::render`](crate::renderer::Renderer::render).
    ///
    /// This flag is automatically set whenever the page content changes but
    /// you can set it directly in case you need to force a repaint.
    pub fn set_needs_paint(&self, needs_paint: bool) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewSetNeedsPaint(self.internal, needs_paint)
        }
    }

    /// Whether or not this View should be repainted during the next call to
    /// [`Renderer::render`](crate::renderer::Renderer::render).
    pub fn needs_paint(&self) -> bool {
        unsafe { self.lib.ultralight().ulViewGetNeedsPaint(self.internal) }
    }

    /// Create an Inspector View to inspect / debug this View locally
    ///
    /// This will only succeed if you have the
    /// inspector assets in your filesystem-- the inspector will
    /// look for `file:///inspector/Main.html` when it loads.
    ///
    /// You must handle [`View::set_create_inspector_view_callback`] so that
    /// the library has a View to display the inspector in. This function will
    /// call the callback only if an inspector view is not currently active.
    pub fn create_local_inspector_view(&self) {
        unsafe {
            self.lib
                .ultralight()
                .ulViewCreateLocalInspectorView(self.internal);
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                self.lib.ultralight().ulDestroyView(self.internal);
            }
        }
    }
}
