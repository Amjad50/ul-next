use crate::config::Config;
use std::ffi::c_void;

pub struct Settings {
    internal: ul_sys::ULSettings,
}

impl Settings {
    pub fn start() -> SettingsBuilder {
        SettingsBuilder::default()
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULSettings {
        self.internal
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroySettings(self.internal);
        }
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    developer_name: Option<String>,
    app_name: Option<String>,
    file_system_path: Option<String>,
    load_shaders_from_filesystem: Option<bool>,
    force_cpu_renderer: Option<bool>,
}

impl SettingsBuilder {
    pub fn developer_name(mut self, developer_name: &str) -> Self {
        self.developer_name = Some(developer_name.to_string());
        self
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = Some(app_name.to_string());
        self
    }

    pub fn file_system_path(mut self, file_system_path: &str) -> Self {
        self.file_system_path = Some(file_system_path.to_string());
        self
    }

    pub fn load_shaders_from_filesystem(mut self, load_shaders_from_filesystem: bool) -> Self {
        self.load_shaders_from_filesystem = Some(load_shaders_from_filesystem);
        self
    }

    pub fn force_cpu_renderer(mut self, force_cpu_renderer: bool) -> Self {
        self.force_cpu_renderer = Some(force_cpu_renderer);
        self
    }

    pub fn build(self) -> Settings {
        let internal = unsafe { ul_sys::ulCreateSettings() };

        set_config_str!(internal, self.developer_name, ulSettingsSetDeveloperName);

        set_config_str!(internal, self.app_name, ulSettingsSetAppName);

        set_config_str!(internal, self.file_system_path, ulSettingsSetFileSystemPath);

        set_config!(
            internal,
            self.load_shaders_from_filesystem,
            ulSettingsSetLoadShadersFromFileSystem
        );

        set_config!(
            internal,
            self.force_cpu_renderer,
            ulSettingsSetForceCPURenderer
        );

        Settings { internal }
    }
}

pub struct Monitor {
    internal: ul_sys::ULMonitor,
}

impl Monitor {
    pub fn get_scale(&self) -> f64 {
        unsafe { ul_sys::ulMonitorGetScale(self.internal) }
    }

    pub fn get_width(&self) -> u32 {
        unsafe { ul_sys::ulMonitorGetWidth(self.internal) }
    }

    pub fn get_height(&self) -> u32 {
        unsafe { ul_sys::ulMonitorGetHeight(self.internal) }
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULMonitor {
        self.internal
    }
}

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

pub struct WindowFlags {
    pub borderless: bool,
    pub titled: bool,
    pub resizable: bool,
    pub maximizable: bool,
    pub hidden: bool,
}

impl WindowFlags {
    fn to_u32(&self) -> u32 {
        let mut n = 0;

        if self.borderless {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Borderless;
        }
        if self.titled {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Titled;
        }
        if self.resizable {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Resizable;
        }
        if self.maximizable {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Maximizable;
        }
        if self.hidden {
            n |= ul_sys::ULWindowFlags_kWindowFlags_Hidden;
        }

        n
    }
}

pub struct Window {
    internal: ul_sys::ULWindow,

    close_callback: Option<Box<Box<dyn FnMut() + 'static>>>,
    resize_callback: Option<Box<Box<dyn FnMut(u32, u32) + 'static>>>,
}

impl Window {
    pub fn screen_width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenWidth(self.internal) }
    }

    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetWidth(self.internal) }
    }

    pub fn screen_height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetScreenHeight(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulWindowGetHeight(self.internal) }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulWindowMoveTo(self.internal, x, y) }
    }

    pub fn move_to_center(&self) {
        unsafe { ul_sys::ulWindowMoveToCenter(self.internal) }
    }

    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionX(self.internal) }
    }

    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulWindowGetPositionY(self.internal) }
    }

    pub fn is_fullscreen(&self) -> bool {
        unsafe { ul_sys::ulWindowIsFullscreen(self.internal) }
    }

    pub fn scale(&self) -> f64 {
        unsafe { ul_sys::ulWindowGetScale(self.internal) }
    }

    pub fn set_title(&self, title: &str) {
        unsafe { ul_sys::ulWindowSetTitle(self.internal, title.as_ptr() as *const i8) }
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe { ul_sys::ulWindowSetCursor(self.internal, cursor as u32) }
    }

    pub fn show(&self) {
        unsafe { ul_sys::ulWindowShow(self.internal) }
    }

    pub fn hide(&self) {
        unsafe { ul_sys::ulWindowHide(self.internal) }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { ul_sys::ulWindowIsVisible(self.internal) }
    }

    pub fn close(&self) {
        unsafe { ul_sys::ulWindowClose(self.internal) }
    }

    pub fn screen_to_pixels(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowScreenToPixels(self.internal, val) }
    }

    pub fn pixels_to_screen(&self, val: i32) -> i32 {
        unsafe { ul_sys::ulWindowPixelsToScreen(self.internal, val) }
    }

    pub fn set_close_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        c_callback! {
            unsafe extern "C" fn window_close_callback(_window: ul_sys::ULWindow);
        }

        // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
        // that can't be cast to a `*const c_void`.
        let mut callback = Box::new(Box::new(callback) as Box<_>);

        // SAFETY: We're passing a pointer to a function that is guaranteed to be valid for the
        // lifetime of the app.
        unsafe {
            ul_sys::ulWindowSetCloseCallback(
                self.internal,
                Some(window_close_callback::<F>),
                &mut *callback as &mut Box<_> as *mut Box<_> as *mut c_void,
            );
        }

        self.close_callback = Some(callback);
    }

    pub fn set_resize_callback<F>(&mut self, callback: F)
    where
        F: FnMut(u32, u32) + 'static,
    {
        c_callback! {
            unsafe extern "C" fn window_resize_callback(_window: ul_sys::ULWindow, width: u32, height:u32): (width: u32, height: u32);
        }

        // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
        // that can't be cast to a `*const c_void`.
        let mut callback = Box::new(Box::new(callback) as Box<_>);

        // SAFETY: We're passing a pointer to a function that is guaranteed to be valid for the
        // lifetime of the app.
        unsafe {
            ul_sys::ulWindowSetResizeCallback(
                self.internal,
                Some(window_resize_callback::<F>),
                &mut *callback as &mut Box<_> as *mut Box<_> as *mut c_void,
            );
        }

        self.resize_callback = Some(callback);
    }

    //pub fn is_accelerated(&self) -> bool {
    //}
    //
    //pub fn render_buff_id(&self) -> u32 {
    //}
    //
    //pub fn draw_surface(&self, surface: &Surface) {
    //}
}

impl Window {
    pub fn create_overlay(&self, width: u32, height: u32, x: i32, y: i32) -> Overlay {
        unsafe {
            let overlay = ul_sys::ulCreateOverlay(self.internal, width, height, x, y);
            let view = View {
                internal: ul_sys::ulOverlayGetView(overlay),
            };
            Overlay {
                internal: overlay,
                view,
            }
        }
    }

    pub fn create_overlay_with_view(&self, view: View, x: i32, y: i32) -> Overlay {
        unsafe {
            let overlay = ul_sys::ulCreateOverlayWithView(self.internal, view.internal, x, y);
            Overlay {
                internal: overlay,
                view,
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyWindow(self.internal);
        }
    }
}

pub struct View {
    internal: ul_sys::ULView,
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyView(self.internal);
        }
    }
}

pub struct Overlay {
    internal: ul_sys::ULOverlay,

    view: View,
}

impl Overlay {
    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn width(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetWidth(self.internal) }
    }

    pub fn height(&self) -> u32 {
        unsafe { ul_sys::ulOverlayGetHeight(self.internal) }
    }

    pub fn x(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetX(self.internal) }
    }

    pub fn y(&self) -> i32 {
        unsafe { ul_sys::ulOverlayGetY(self.internal) }
    }

    pub fn is_hidden(&self) -> bool {
        unsafe { ul_sys::ulOverlayIsHidden(self.internal) }
    }

    pub fn show(&self) {
        unsafe { ul_sys::ulOverlayShow(self.internal) }
    }

    pub fn hide(&self) {
        unsafe { ul_sys::ulOverlayHide(self.internal) }
    }

    pub fn has_focus(&self) -> bool {
        unsafe { ul_sys::ulOverlayHasFocus(self.internal) }
    }

    pub fn focus(&self) {
        unsafe { ul_sys::ulOverlayFocus(self.internal) }
    }

    pub fn unfocus(&self) {
        unsafe { ul_sys::ulOverlayUnfocus(self.internal) }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        unsafe { ul_sys::ulOverlayMoveTo(self.internal, x, y) }
    }

    pub fn resize(&self, width: u32, height: u32) {
        unsafe { ul_sys::ulOverlayResize(self.internal, width, height) }
    }

    //pub fn need_repaint(&self) -> bool {
    //}
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyOverlay(self.internal);
        }
    }
}

pub struct App {
    config: Config,
    settings: Settings,

    monitor: Monitor,

    internal: ul_sys::ULApp,

    update_callback: Option<Box<Box<dyn FnMut() + 'static>>>,
}

impl App {
    pub fn new(settings: Option<Settings>, config: Option<Config>) -> Self {
        let config = match config {
            Some(config) => config,
            None => Config::start().build(),
        };

        let settings = match settings {
            Some(settings) => settings,
            None => Settings::start().build(),
        };

        unsafe {
            let app_internal = ul_sys::ulCreateApp(settings.to_ul(), config.to_ul());

            let monitor = Monitor {
                internal: ul_sys::ulAppGetMainMonitor(app_internal),
            };

            Self {
                config,
                settings,
                internal: app_internal,
                monitor,
                update_callback: None,
            }
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn main_monitor(&self) -> &Monitor {
        &self.monitor
    }

    pub fn is_running(&self) -> bool {
        unsafe { ul_sys::ulAppIsRunning(self.internal) }
    }

    //pub fn renderer(&self) -> bool {
    //}

    pub fn set_update_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        c_callback! {
            unsafe extern "C" fn app_update_callback();
        }

        // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
        // that can't be cast to a `*const c_void`.
        let mut callback = Box::new(Box::new(callback) as Box<_>);

        // SAFETY: We're passing a pointer to a function that is guaranteed to be valid for the
        // lifetime of the app.
        unsafe {
            ul_sys::ulAppSetUpdateCallback(
                self.internal,
                Some(app_update_callback::<F>),
                &mut *callback as &mut Box<_> as *mut Box<_> as *mut c_void,
            );
        }

        self.update_callback = Some(callback);
    }

    pub fn run(&self) {
        unsafe { ul_sys::ulAppRun(self.internal) }
    }

    pub fn quit(&self) {
        unsafe { ul_sys::ulAppQuit(self.internal) }
    }

    pub fn create_window(
        &self,
        width: u32,
        height: u32,
        fullscreen: bool,
        window_flags: WindowFlags,
    ) -> Window {
        let internal = unsafe {
            ul_sys::ulCreateWindow(
                self.monitor.internal,
                width,
                height,
                fullscreen,
                window_flags.to_u32(),
            )
        };

        Window {
            internal,
            close_callback: None,
            resize_callback: None,
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyApp(self.internal);
        }
    }
}

#[test]
fn test_app() {
    // builds relative path from the exe location to the current location
    // The library doesn't support absolute paths
    let mut path =
        std::path::PathBuf::from_iter(std::env::current_exe().unwrap().components().map(|_| ".."))
            .to_string_lossy()
            .to_string();
    path.push_str(&std::env::current_dir().unwrap().to_string_lossy());

    // set the file system path to the current location, to access resources
    let app = App::new(
        Some(Settings::start().file_system_path(&path).build()),
        None,
    );

    // we must assign the window to a variable, otherwise it will be dropped
    // TODO: maybe we should keep the window in the app?
    let mut window = app.create_window(
        1280,
        720,
        false,
        WindowFlags {
            borderless: false,
            titled: true,
            resizable: true,
            maximizable: true,
            hidden: false,
        },
    );

    let app = std::rc::Rc::new(app);
    let app_clone = app.clone();
    window.set_close_callback(move || {
        assert!(app_clone.is_running());

        println!("close");
        app_clone.quit();
    });

    window.set_resize_callback(|width, height| {
        println!("resize {} {}", width, height);
    });

    // main loop
    app.run();
}
