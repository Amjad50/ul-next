use crate::{config::Config, utils::c_callback};
use std::{ffi::c_void, panic};

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
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyWindow(self.internal);
        }
    }
}

pub struct App {
    config: Config,
    settings: Settings,

    monitor: Monitor,

    internal: ul_sys::ULApp,

    update_callback: Option<Box<Box<dyn FnMut() + 'static + Send>>>,
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
        F: FnMut() + 'static + Send + panic::RefUnwindSafe,
    {
        // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
        // that can't be cast to a `*const c_void`.
        let mut callback = Box::new(Box::new(callback) as Box<_>);

        // SAFETY: We're passing a pointer to a function that is guaranteed to be valid for the
        // lifetime of the app.
        unsafe {
            ul_sys::ulAppSetUpdateCallback(
                self.internal,
                Some(c_callback::<F>),
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

        Window { internal }
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
    let mut app = App::new(
        Some(Settings::start().file_system_path(&path).build()),
        None,
    );

    let mut i = 0;
    app.set_update_callback(move || {
        i += 1;
        println!("update {}", i);
    });

    // we must assign the window to a variable, otherwise it will be dropped
    // TODO: maybe we should keep the window in the app?
    let window = app.create_window(
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

    // this will never return
    app.run();

    app.quit();
}
