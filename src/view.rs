use crate::{
    event::{KeyEvent, MouseEvent, ScrollEvent},
    render_target::RenderTarget,
    session::Session,
    string::UlString,
};

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

    pub fn render_target(&self) -> RenderTarget {
        unsafe { RenderTarget::from(ul_sys::ulViewGetRenderTarget(self.internal)) }
    }

    //pub fn surface(&self) -> Surface {
    //    unsafe { Surface::from_raw(ul_sys::ulViewGetSurface(self.internal)) }
    //}

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

    pub fn evaluate_script(&self, script: &str) -> String {
        unsafe {
            let ul_script_string = UlString::from_str(script);
            // a dummy value, it will be replaced by the actual result
            let mut exception_string = 1 as ul_sys::ULString;
            let result_string = ul_sys::ulViewEvaluateScript(
                self.internal,
                ul_script_string.to_ul(),
                &mut exception_string as _,
            );
            UlString::copy_raw_to_string(result_string)
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
