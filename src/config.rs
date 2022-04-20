pub enum FaceWinding {
    Clockwise = ul_sys::ULFaceWinding_kFaceWinding_Clockwise as isize,
    CounterClockwise = ul_sys::ULFaceWinding_kFaceWindow_CounterClockwise as isize,
}

pub enum FontHinting {
    Smooth = ul_sys::ULFontHinting_kFontHinting_Smooth as isize,
    Normal = ul_sys::ULFontHinting_kFontHinting_Normal as isize,
    Monochrome = ul_sys::ULFontHinting_kFontHinting_Monochrome as isize,
}

pub struct Config {
    internal: ul_sys::ULConfig,
}

impl Config {
    pub fn start() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn to_ul(&self) -> ul_sys::ULConfig {
        self.internal
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyConfig(self.internal);
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    cache_path: Option<String>,
    //resource_path_prefix: Option<String>,
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
    //num_renderer_threads: Option<u32>,
    //max_update_time: Option<f64>,
}

impl ConfigBuilder {
    pub fn cache_path(mut self, path: &str) -> Self {
        self.cache_path = Some(path.to_string());
        self
    }

    pub fn face_winding(mut self, winding: FaceWinding) -> Self {
        self.face_winding = Some(winding);
        self
    }

    pub fn font_hinting(mut self, hinting: FontHinting) -> Self {
        self.font_hinting = Some(hinting);
        self
    }

    pub fn font_gamma(mut self, gamma: f64) -> Self {
        self.font_gamma = Some(gamma);
        self
    }

    pub fn user_stylesheet(mut self, path: &str) -> Self {
        self.user_stylesheet = Some(path.to_string());
        self
    }

    pub fn force_repaint(mut self, force: bool) -> Self {
        self.force_repaint = Some(force);
        self
    }

    pub fn animation_timer_delay(mut self, delay: f64) -> Self {
        self.animation_timer_delay = Some(delay);
        self
    }

    pub fn scroll_timer_delay(mut self, delay: f64) -> Self {
        self.scroll_timer_delay = Some(delay);
        self
    }

    pub fn recycle_delay(mut self, delay: f64) -> Self {
        self.recycle_delay = Some(delay);
        self
    }

    pub fn memory_cache_size(mut self, size: u32) -> Self {
        self.memory_cache_size = Some(size);
        self
    }

    pub fn page_cache_size(mut self, size: u32) -> Self {
        self.page_cache_size = Some(size);
        self
    }

    pub fn override_ram_size(mut self, size: u32) -> Self {
        self.override_ram_size = Some(size);
        self
    }

    pub fn min_large_heap_size(mut self, size: u32) -> Self {
        self.min_large_heap_size = Some(size);
        self
    }

    pub fn min_small_heap_size(mut self, size: u32) -> Self {
        self.min_small_heap_size = Some(size);
        self
    }

    pub fn build(self) -> Config {
        let internal = unsafe { ul_sys::ulCreateConfig() };

        set_config_str!(internal, self.cache_path, ulConfigSetCachePath);
        //set_config_str!(
        //    internal,
        //    self.resource_path_prefix,
        //    ulConfigSetResourcePathPrefix
        //);
        set_config!(
            internal,
            self.face_winding.map(|x| x as u32),
            ulConfigSetFaceWinding
        );
        set_config!(
            internal,
            self.font_hinting.map(|x| x as u32),
            ulConfigSetFontHinting
        );
        set_config!(internal, self.font_gamma, ulConfigSetFontGamma);
        set_config_str!(internal, self.user_stylesheet, ulConfigSetUserStylesheet);
        set_config!(internal, self.force_repaint, ulConfigSetForceRepaint);
        set_config!(
            internal,
            self.animation_timer_delay,
            ulConfigSetAnimationTimerDelay
        );
        set_config!(
            internal,
            self.scroll_timer_delay,
            ulConfigSetScrollTimerDelay
        );
        set_config!(internal, self.recycle_delay, ulConfigSetRecycleDelay);
        set_config!(internal, self.memory_cache_size, ulConfigSetMemoryCacheSize);
        set_config!(internal, self.page_cache_size, ulConfigSetPageCacheSize);
        set_config!(internal, self.override_ram_size, ulConfigSetOverrideRAMSize);
        set_config!(
            internal,
            self.min_large_heap_size,
            ulConfigSetMinLargeHeapSize
        );
        set_config!(
            internal,
            self.min_small_heap_size,
            ulConfigSetMinSmallHeapSize
        );
        //set_config!(
        //    internal,
        //    self.num_renderer_threads,
        //    ulConfigSetNumRendererThreads
        //);
        //set_config!(internal, self.max_update_time, ulConfigSetMaxUpdateTime);

        Config { internal }
    }
}
