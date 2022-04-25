use crate::{
    event::{KeyEvent, MouseEvent, ScrollEvent},
    rect::Rect,
    render_target::RenderTarget,
    session::Session,
    string::UlString,
    surface::Surface,
    window::Cursor,
};

#[derive(Clone, Copy, Debug)]
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

pub struct ViewConfig {
    internal: ul_sys::ULViewConfig,
}

impl ViewConfig {
    pub fn start() -> ViewConfigBuilder {
        ViewConfigBuilder::default()
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULViewConfig {
        self.internal
    }
}

impl Drop for ViewConfig {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyViewConfig(self.internal);
        }
    }
}

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
}

impl ViewConfigBuilder {
    pub fn is_accelerated(mut self, is_accelerated: bool) -> Self {
        self.is_accelerated = Some(is_accelerated);
        self
    }

    pub fn is_transparent(mut self, is_transparent: bool) -> Self {
        self.is_transparent = Some(is_transparent);
        self
    }

    pub fn initial_device_scale(mut self, initial_device_scale: f64) -> Self {
        self.initial_device_scale = Some(initial_device_scale);
        self
    }

    pub fn initial_focus(mut self, initial_focus: bool) -> Self {
        self.initial_focus = Some(initial_focus);
        self
    }

    pub fn enable_images(mut self, enable_images: bool) -> Self {
        self.enable_images = Some(enable_images);
        self
    }

    pub fn enable_javascript(mut self, enable_javascript: bool) -> Self {
        self.enable_javascript = Some(enable_javascript);
        self
    }

    pub fn font_family_standard(mut self, font_family_standard: &str) -> Self {
        self.font_family_standard = Some(font_family_standard.to_string());
        self
    }

    pub fn font_family_fixed(mut self, font_family_fixed: &str) -> Self {
        self.font_family_fixed = Some(font_family_fixed.to_string());
        self
    }

    pub fn font_family_serif(mut self, font_family_serif: &str) -> Self {
        self.font_family_serif = Some(font_family_serif.to_string());
        self
    }

    pub fn font_family_sans_serif(mut self, font_family_sans_serif: &str) -> Self {
        self.font_family_sans_serif = Some(font_family_sans_serif.to_string());
        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn build(self) -> ViewConfig {
        let internal = unsafe { ul_sys::ulCreateViewConfig() };

        set_config!(internal, self.is_accelerated, ulViewConfigSetIsAccelerated);
        set_config!(internal, self.is_transparent, ulViewConfigSetIsTransparent);
        set_config!(
            internal,
            self.initial_device_scale,
            ulViewConfigSetInitialDeviceScale
        );
        set_config!(internal, self.initial_focus, ulViewConfigSetInitialFocus);
        set_config!(internal, self.enable_images, ulViewConfigSetEnableImages);
        set_config!(
            internal,
            self.enable_javascript,
            ulViewConfigSetEnableJavaScript
        );
        set_config_str!(
            internal,
            self.font_family_standard,
            ulViewConfigSetFontFamilyStandard
        );
        set_config_str!(
            internal,
            self.font_family_fixed,
            ulViewConfigSetFontFamilyFixed
        );
        set_config_str!(
            internal,
            self.font_family_serif,
            ulViewConfigSetFontFamilySerif
        );
        set_config_str!(
            internal,
            self.font_family_sans_serif,
            ulViewConfigSetFontFamilySansSerif
        );
        set_config_str!(internal, self.user_agent, ulViewConfigSetUserAgent);

        ViewConfig { internal }
    }
}

pub struct View {
    internal: ul_sys::ULView,
    need_to_destroy: bool,
}

impl View {
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULView) -> Self {
        Self {
            internal: raw,
            need_to_destroy: false,
        }
    }

    pub(crate) unsafe fn create(
        renderer: ul_sys::ULRenderer,
        width: u32,
        height: u32,
        view_config: &ViewConfig,
        session: Option<Session>,
    ) -> Self {
        let internal = ul_sys::ulCreateView(
            renderer,
            width,
            height,
            view_config.to_ul(),
            session.map(|s| s.to_ul()).unwrap_or(std::ptr::null_mut()),
        );

        Self {
            internal,
            need_to_destroy: true,
        }
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULView {
        self.internal
    }
}

impl View {
    pub fn url(&self) -> String {
        unsafe {
            let url_string = ul_sys::ulViewGetURL(self.internal);
            UlString::copy_raw_to_string(url_string)
        }
    }

    pub fn title(&self) -> String {
        unsafe {
            let title_string = ul_sys::ulViewGetTitle(self.internal);
            UlString::copy_raw_to_string(title_string)
        }
    }

    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulViewGetWidth(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulViewGetHeight(self.internal) }
    }

    pub fn device_scale(&self) -> f64 {
        unsafe { ul_sys::ulViewGetDeviceScale(self.internal) }
    }

    pub fn set_device_scale(&self, scale: f64) {
        unsafe { ul_sys::ulViewSetDeviceScale(self.internal, scale) }
    }

    pub fn is_accelerated(&self) -> bool {
        unsafe { ul_sys::ulViewIsAccelerated(self.internal) }
    }

    pub fn is_transparent(&self) -> bool {
        unsafe { ul_sys::ulViewIsTransparent(self.internal) }
    }

    pub fn is_loading(&self) -> bool {
        unsafe { ul_sys::ulViewIsLoading(self.internal) }
    }

    /// Get the RenderTarget for the View.
    ///
    ///  Only valid when the view is accelerated.
    pub fn render_target(&self) -> Option<RenderTarget> {
        if self.is_accelerated() {
            Some(unsafe { RenderTarget::from(ul_sys::ulViewGetRenderTarget(self.internal)) })
        } else {
            None
        }
    }

    /// Get the Surface for the View (native pixel buffer container).
    ///
    /// This is only valid when the view is not accelerated.
    ///
    /// (Will return a [`None`] when the GPU renderer is enabled.)
    ///
    /// TODO:
    /// The default Surface is BitmapSurface but you can provide your own Surface implementation
    /// via [`Platform::set_surface_definition`].
    ///
    /// TODO:
    /// When using the default Surface, you can retrieve the underlying bitmap by casting
    /// ULSurface to ULBitmapSurface and calling ulBitmapSurfaceGetBitmap().
    pub fn surface(&self) -> Option<Surface> {
        if !self.is_accelerated() {
            unsafe {
                let surface = ul_sys::ulViewGetSurface(self.internal);
                if surface.is_null() {
                    None
                } else {
                    Some(Surface::from_raw(surface))
                }
            }
        } else {
            None
        }
    }

    pub fn load_html(&self, html: &str) {
        unsafe {
            let ul_string = UlString::from_str(html);
            ul_sys::ulViewLoadHTML(self.internal, ul_string.to_ul());
        }
    }

    pub fn load_url(&self, url: &str) {
        unsafe {
            let ul_string = UlString::from_str(url);
            ul_sys::ulViewLoadURL(self.internal, ul_string.to_ul());
        }
    }

    pub fn resize(&self, width: u32, height: u32) {
        unsafe {
            ul_sys::ulViewResize(self.internal, width, height);
        }
    }

    //pub fn lock_js_context(&self) -> JsContext {
    //  ul_string::ulViewLockJSContext(self.internal)
    //}

    //pub fn javascript_vm(&self) {
    //}

    pub fn evaluate_script(&self, script: &str) -> Result<String, String> {
        unsafe {
            let ul_script_string = UlString::from_str(script);
            // a dummy value, it will be replaced by the actual result
            let mut exception_string = 1 as ul_sys::ULString;
            let result_string = ul_sys::ulViewEvaluateScript(
                self.internal,
                ul_script_string.to_ul(),
                &mut exception_string as _,
            );

            let has_exception = !ul_sys::ulStringIsEmpty(exception_string);
            if has_exception {
                let exception_string = UlString::copy_raw_to_string(exception_string);
                Err(exception_string)
            } else {
                let result_string = UlString::copy_raw_to_string(result_string);
                Ok(result_string)
            }
        }
    }

    pub fn can_go_back(&self) -> bool {
        unsafe { ul_sys::ulViewCanGoBack(self.internal) }
    }

    pub fn can_go_forward(&self) -> bool {
        unsafe { ul_sys::ulViewCanGoForward(self.internal) }
    }

    pub fn go_back(&self) {
        unsafe { ul_sys::ulViewGoBack(self.internal) }
    }

    pub fn go_forward(&self) {
        unsafe { ul_sys::ulViewGoForward(self.internal) }
    }

    pub fn go_to_history_offset(&self, offset: i32) {
        unsafe { ul_sys::ulViewGoToHistoryOffset(self.internal, offset) }
    }

    pub fn reload(&self) {
        unsafe { ul_sys::ulViewReload(self.internal) }
    }

    pub fn stop(&self) {
        unsafe { ul_sys::ulViewStop(self.internal) }
    }

    pub fn focus(&self) {
        unsafe { ul_sys::ulViewFocus(self.internal) }
    }

    pub fn unfocus(&self) {
        unsafe { ul_sys::ulViewUnfocus(self.internal) }
    }

    pub fn has_focus(&self) -> bool {
        unsafe { ul_sys::ulViewHasFocus(self.internal) }
    }

    pub fn has_input_focus(&self) -> bool {
        unsafe { ul_sys::ulViewHasInputFocus(self.internal) }
    }

    pub fn fire_key_event(&self, key_event: KeyEvent) {
        unsafe { ul_sys::ulViewFireKeyEvent(self.internal, key_event.to_ul()) }
    }

    pub fn fire_mouse_event(&self, mouse_event: MouseEvent) {
        unsafe { ul_sys::ulViewFireMouseEvent(self.internal, mouse_event.to_ul()) }
    }

    pub fn fire_scroll_event(&self, scroll_event: ScrollEvent) {
        unsafe { ul_sys::ulViewFireScrollEvent(self.internal, scroll_event.to_ul()) }
    }

    // looking at the CPP header, the strings seems to be references
    // but the C headers doesn't say we must not destroy them.
    // For now we don't destroy.
    //  TODO: check if we don't need to destroy them
    set_callback! {
        pub fn set_change_title_callback(&self, callback: FnMut(view: &View, title: String)) :
           ulViewSetChangeTitleCallback(ul_view: ul_sys::ULView, ul_title: ul_sys::ULString) {
               let view = &View::from_raw(ul_view);
               let title = UlString::copy_raw_to_string(ul_title);
        }
    }

    set_callback! {
        pub fn set_change_url_callback(&self, callback: FnMut(view: &View, url: String)) :
           ulViewSetChangeURLCallback(ul_view: ul_sys::ULView, ul_url: ul_sys::ULString) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
        }
    }

    set_callback! {
        pub fn set_change_tooltip_callback(&self, callback: FnMut(view: &View, tooltip: String)) :
           ulViewSetChangeTooltipCallback(ul_view: ul_sys::ULView, ul_tooltip: ul_sys::ULString) {
               let view = &View::from_raw(ul_view);
               let tooltip = UlString::copy_raw_to_string(ul_tooltip);
        }
    }

    set_callback! {
        pub fn set_change_cursor_callback(&self, callback: FnMut(view: &View, cursor: Cursor)) :
           ulViewSetChangeCursorCallback(ul_view: ul_sys::ULView, ul_cursor: ul_sys::ULCursor) {
               let view = &View::from_raw(ul_view);
               // TODO: handle strings
               let cursor = Cursor::try_from(ul_cursor).unwrap();
        }
    }

    set_callback! {
        pub fn set_add_console_message_callback(&self, callback: FnMut(
                view: &View,
                message_source: ConsoleMessageSource,
                message_level: ConsoleMessageLevel,
                message: String,
                line_number:u32,
                column_number:u32,
                source_id: String)) :
           ulViewSetAddConsoleMessageCallback(
               ul_view: ul_sys::ULView,
               ul_message_source: ul_sys::ULMessageSource,
               ul_message_level: ul_sys::ULMessageLevel,
               ul_message: ul_sys::ULString,
               line_number: u32,
               column_number :u32,
               ul_source_id: ul_sys::ULString
            ) {
               let view = &View::from_raw(ul_view);
               let message_source = ConsoleMessageSource::try_from(ul_message_source).unwrap();
               let message_level = ConsoleMessageLevel::try_from(ul_message_level).unwrap();
               let message = UlString::copy_raw_to_string(ul_message);
               let source_id = UlString::copy_raw_to_string(ul_source_id);
        }
    }

    set_callback! {
        /// Set callback for when the page wants to create a new View.
        ///
        /// This is usually the result of a user clicking a link with target="_blank" or by JavaScript
        /// calling window.open(url).
        ///
        /// To allow creation of these new Views, you should create a new View in this callback, resize it
        /// to your container, and return it. You are responsible for displaying the returned View.
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
           ulViewSetCreateChildViewCallback(
               ul_view: ul_sys::ULView,
               ul_opener_url: ul_sys::ULString,
               ul_target_url: ul_sys::ULString,
               is_popup: bool,
               ul_popup_rect: ul_sys::ULIntRect
            ) -> ul_sys::ULView {
               let view = &View::from_raw(ul_view);
               let opener_url = UlString::copy_raw_to_string(ul_opener_url);
               let target_url = UlString::copy_raw_to_string(ul_target_url);
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
        pub fn set_begin_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
           ulViewSetBeginLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
        }
    }

    set_callback! {
        pub fn set_finish_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
           ulViewSetFinishLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
        }
    }

    set_callback! {
        pub fn set_fail_loading_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String,
                description: String,
                error_domain: String,
                error_code: i32)) :
           ulViewSetFailLoadingCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString,
               ul_description: ul_sys::ULString,
               ul_error_domain: ul_sys::ULString,
               error_code: i32
            ) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
               let description = UlString::copy_raw_to_string(ul_description);
               let error_domain = UlString::copy_raw_to_string(ul_error_domain);
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
        pub fn set_window_object_ready_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
           ulViewSetWindowObjectReadyCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
        }
    }

    set_callback! {
        pub fn set_dom_ready_callback(&self, callback: FnMut(
                view: &View,
                frame_id: u64,
                is_main_frame: bool,
                url: String)) :
           ulViewSetDOMReadyCallback(
               ul_view: ul_sys::ULView,
               frame_id: u64,
               is_main_frame: bool,
               ul_url: ul_sys::ULString
            ) {
               let view = &View::from_raw(ul_view);
               let url = UlString::copy_raw_to_string(ul_url);
        }
    }

    set_callback! {
        pub fn set_update_history_callback(&self, callback: FnMut(view: &View)) :
           ulViewSetUpdateHistoryCallback(ul_view: ul_sys::ULView) {
               let view = &View::from_raw(ul_view);
        }
    }

    pub fn set_needs_paint(&self, needs_paint: bool) {
        unsafe { ul_sys::ulViewSetNeedsPaint(self.internal, needs_paint) }
    }

    pub fn needs_paint(&self) -> bool {
        unsafe { ul_sys::ulViewGetNeedsPaint(self.internal) }
    }

    pub fn create_inspector_view(&self) -> View {
        unsafe {
            let inspector_view = ul_sys::ulViewCreateInspectorView(self.internal);
            // we need to destroy the view when its dropped, its now owned by anyone
            View {
                internal: inspector_view,
                need_to_destroy: true,
            }
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                ul_sys::ulDestroyView(self.internal);
            }
        }
    }
}
