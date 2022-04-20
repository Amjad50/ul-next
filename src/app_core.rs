use crate::config::Config;

struct Settings {
    internal: ul_sys::ULSettings,
}

impl Settings {
    pub fn start() -> SettingsBuilder {
        SettingsBuilder::default()
    }

    pub fn to_ul(&self) -> ul_sys::ULSettings {
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
struct SettingsBuilder {
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

struct Monitor {
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

    pub fn to_ul(&self) -> ul_sys::ULMonitor {
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

struct Window {
    internal: ul_sys::ULWindow,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyWindow(self.internal);
        }
    }
}

struct App {
    config: Config,
    settings: Settings,

    monitor: Monitor,

    internal: ul_sys::ULApp,
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
                //phantom: PhantomData,
            }
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn main_monitor(&self) -> &Monitor {
        &self.monitor
    }

    pub fn is_running(&self) -> bool {
        unsafe { ul_sys::ulAppIsRunning(self.internal) }
    }

    //pub fn renderer(&self) -> bool {
    //}

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
    let app = App::new(
        Some(Settings::start().file_system_path(&path).build()),
        None,
    );

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
