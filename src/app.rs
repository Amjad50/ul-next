//! An `App` component to create GUI applications with `Ultralight`.
//!
//! The `App` component of `Ultralight` allows you to create a GUI application
//! that uses GPU rendering to render web pages.
//!
//! If you want to have access to the inner parts of the `Ultralight` engine,
//! have access to the textures to integrate into your game/application, check
//! [`Renderer`](crate::renderer::Renderer) where you can implement your own
//! [`GpuDriver`](crate::gpu_driver::GpuDriver) and integrate it with your project.
use crate::config::Config;
use crate::renderer::Renderer;
use crate::window::{Window, WindowFlags};

/// Settings specific for the [`App`].
pub struct Settings {
    internal: ul_sys::ULSettings,
}

impl Settings {
    /// Starts the building process for the [`Settings`] struct. returns a builder
    /// which can be used to configure the settings.
    pub fn start() -> SettingsBuilder {
        SettingsBuilder::default()
    }

    /// Returns the underlying [`ul_sys::ULSettings`] struct, to be used locally for
    /// calling the underlying C API.
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
/// Builder for the [`Settings`] struct.
pub struct SettingsBuilder {
    developer_name: Option<String>,
    app_name: Option<String>,
    file_system_path: Option<String>,
    load_shaders_from_filesystem: Option<bool>,
    force_cpu_renderer: Option<bool>,
}

impl SettingsBuilder {
    /// The name of the developer of this app.
    ///
    /// This is used to generate a unique path to store local application data
    /// on the user's machine.
    pub fn developer_name(mut self, developer_name: &str) -> Self {
        self.developer_name = Some(developer_name.to_string());
        self
    }

    /// The name of this app.
    ///
    /// This is used to generate a unique path to store local application data
    /// on the user's machine.
    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = Some(app_name.to_string());
        self
    }

    /// The root file path for our file system. You should set this to the
    /// relative path where all of your app data is.
    ///
    /// This will be used to resolve all file URLs, eg `file:///page.html`.
    ///
    /// This relative path is resolved using the following logic:
    ///     - Windows: relative to the executable path
    ///     - Linux:   relative to the executable path
    ///     - macOS:   relative to `YourApp.app/Contents/Resources/`
    pub fn file_system_path(mut self, file_system_path: &str) -> Self {
        self.file_system_path = Some(file_system_path.to_string());
        self
    }

    /// Whether or not we should load and compile shaders from the file system
    /// (eg, from the /shaders/ path, relative to [`file_system_path`](Self::file_system_path)).
    ///
    /// If this is false (the default), we will instead load pre-compiled shaders
    /// from memory which speeds up application startup time.
    pub fn load_shaders_from_filesystem(mut self, load_shaders_from_filesystem: bool) -> Self {
        self.load_shaders_from_filesystem = Some(load_shaders_from_filesystem);
        self
    }

    /// We try to use the GPU renderer when a compatible GPU is detected.
    ///
    /// Set this to true to force the engine to always use the CPU renderer.
    pub fn force_cpu_renderer(mut self, force_cpu_renderer: bool) -> Self {
        self.force_cpu_renderer = Some(force_cpu_renderer);
        self
    }

    /// Builds the [`Settings`] struct using the settings configured in this builder.
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

/// Monitor struct, represents a platform monitor.
pub struct Monitor {
    // This is managed by the `App`, so we don't need to free it.
    internal: ul_sys::ULMonitor,
}

impl Monitor {
    /// Get the DPI scale (1.0 = 100%)
    pub fn get_scale(&self) -> f64 {
        unsafe { ul_sys::ulMonitorGetScale(self.internal) }
    }

    /// Get the width of the monitor.
    pub fn get_width(&self) -> u32 {
        unsafe { ul_sys::ulMonitorGetWidth(self.internal) }
    }

    /// Get the height of the monitor.
    pub fn get_height(&self) -> u32 {
        unsafe { ul_sys::ulMonitorGetHeight(self.internal) }
    }
}

/// Main application struct.
pub struct App {
    settings: Settings,

    monitor: Monitor,
    renderer: Renderer,

    internal: ul_sys::ULApp,
}

impl App {
    // TODO: the C++ library creates a singleton and stores the object globaly
    //       should we do the same and return a reference only?
    /// Creates a new application instance.
    ///
    /// # Arguments
    /// * `settings` - The settings to customize the app runtime behaviour.
    /// * `config` - Options for `Ultralight` [`Renderer`](crate::renderer::Renderer).
    ///
    /// Leaving `settings` or `config` as `None` will use the default settings/
    /// config.
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
            let renderer = Renderer::from_raw(ul_sys::ulAppGetRenderer(app_internal));

            Self {
                settings,
                internal: app_internal,
                monitor,
                renderer,
            }
        }
    }

    // TODO: the `Settings` struct is useless since we can't access the settings
    //       fields from the CAPI. so either remove this or find a solution.
    /// Get the settings of the app.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get the main monitor of the app.
    pub fn main_monitor(&self) -> &Monitor {
        &self.monitor
    }

    /// Whether or not the app is running.
    pub fn is_running(&self) -> bool {
        unsafe { ul_sys::ulAppIsRunning(self.internal) }
    }

    /// Get the underlying [`Renderer`](crate::renderer::Renderer) instance.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    set_callback! {
        /// Set a callback to be called whenever the App updates.
        /// You should update all app logic here.
        ///
        /// This event is fired right before the run loop calls
        /// [`Renderer::update`](crate::renderer::Renderer::update) and
        /// [`Renderer::render`](crate::renderer::Renderer::render).
        pub fn set_update_callback(&self, callback: FnMut()) :
            ulAppSetUpdateCallback() {
        }
    }

    /// Start the main loop.
    pub fn run(&self) {
        unsafe { ul_sys::ulAppRun(self.internal) }
    }

    /// Stop the main loop.
    pub fn quit(&self) {
        unsafe { ul_sys::ulAppQuit(self.internal) }
    }

    /// Create a new window.
    ///
    /// # Arguments
    /// * `width` - The width of the window.
    /// * `height` - The height of the window.
    /// * `fullscreen` - Whether or not the window should be fullscreen.
    /// * `window_flags` - Various [`WindowFlags`].
    ///
    /// The window will be shown by default unless [`WindowFlags::hidden`] was set.
    ///
    /// The window will be closed automatically if the object is dropped.
    pub fn create_window(
        &self,
        width: u32,
        height: u32,
        fullscreen: bool,
        window_flags: WindowFlags,
    ) -> Window {
        unsafe {
            Window::create(
                self.monitor.internal,
                width,
                height,
                fullscreen,
                window_flags,
            )
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
