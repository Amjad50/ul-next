//! An `App` component to create GUI applications with `Ultralight`.
//!
//! The `App` component of `Ultralight` allows you to create a GUI application
//! that uses GPU rendering to render web pages.
//!
//! If you want to have access to the inner parts of the `Ultralight` engine,
//! have access to the textures to integrate into your game/application, check
//! [`Renderer`] where you can implement your own
//! [`GpuDriver`](crate::gpu_driver::GpuDriver) and integrate it with your project.
use std::sync::Arc;

use crate::{
    config::Config,
    error::CreationError,
    renderer::Renderer,
    window::{Window, WindowFlags},
    Library,
};

/// Settings specific for the [`App`].
pub struct Settings {
    lib: Arc<Library>,
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
            self.lib.appcore().ulDestroySettings(self.internal);
        }
    }
}

#[derive(Default)]
/// Builder for the [`Settings`] struct.
pub struct SettingsBuilder {
    developer_name: Option<String>,
    app_name: Option<String>,
    filesystem_path: Option<String>,
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
    pub fn filesystem_path(mut self, filesystem_path: &str) -> Self {
        self.filesystem_path = Some(filesystem_path.to_string());
        self
    }

    /// Whether or not we should load and compile shaders from the file system
    /// (eg, from the /shaders/ path, relative to [`filesystem_path`](Self::filesystem_path)).
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
    ///
    /// Return [`None`] if failed to create [`Settings`].
    pub fn build(self, lib: Arc<Library>) -> Option<Settings> {
        let internal = unsafe { lib.appcore().ulCreateSettings() };

        if internal.is_null() {
            return None;
        }

        set_config_str!(
            internal,
            self.developer_name,
            lib.appcore().ulSettingsSetDeveloperName
        );

        set_config_str!(internal, self.app_name, lib.appcore().ulSettingsSetAppName);

        set_config_str!(
            internal,
            self.filesystem_path,
            lib.appcore().ulSettingsSetFileSystemPath
        );

        set_config!(
            internal,
            self.load_shaders_from_filesystem,
            lib.appcore().ulSettingsSetLoadShadersFromFileSystem
        );

        set_config!(
            internal,
            self.force_cpu_renderer,
            lib.appcore().ulSettingsSetForceCPURenderer
        );

        Some(Settings { lib, internal })
    }
}

/// Monitor struct, represents a platform monitor.
pub struct Monitor {
    lib: Arc<Library>,
    // This is managed by the `App`, so we don't need to free it.
    internal: ul_sys::ULMonitor,
}

impl Monitor {
    /// Get the DPI scale (1.0 = 100%)
    pub fn get_scale(&self) -> f64 {
        unsafe { self.lib.appcore().ulMonitorGetScale(self.internal) }
    }

    /// Get the width of the monitor.
    pub fn get_width(&self) -> u32 {
        unsafe { self.lib.appcore().ulMonitorGetWidth(self.internal) }
    }

    /// Get the height of the monitor.
    pub fn get_height(&self) -> u32 {
        unsafe { self.lib.appcore().ulMonitorGetHeight(self.internal) }
    }
}

/// Main application struct.
pub struct App {
    lib: Arc<Library>,
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
    /// * `config` - Options for `Ultralight` [`Renderer`].
    ///
    /// Leaving `settings` or `config` as `None` will use the default settings/
    /// config.
    ///
    /// Returns [`None`] if the application could not be created.
    pub fn new(
        lib: Arc<Library>,
        settings: Option<Settings>,
        config: Option<Config>,
    ) -> Result<Self, CreationError> {
        let config = match config {
            Some(config) => config,
            None => Config::start()
                .build(lib.clone())
                .ok_or(CreationError::NullReference)?,
        };

        let settings = match settings {
            Some(settings) => settings,
            None => Settings::start()
                .build(lib.clone())
                .ok_or(CreationError::NullReference)?,
        };

        unsafe {
            let app_internal = lib.appcore().ulCreateApp(settings.to_ul(), config.to_ul());
            if app_internal.is_null() {
                return Err(CreationError::NullReference);
            }

            let monitor = Monitor {
                lib: lib.clone(),
                internal: lib.appcore().ulAppGetMainMonitor(app_internal),
            };
            if monitor.internal.is_null() {
                lib.appcore().ulDestroyApp(app_internal);
                return Err(CreationError::NullReference);
            }
            let renderer_raw = lib.appcore().ulAppGetRenderer(app_internal);
            if let Ok(renderer) = Renderer::from_raw(lib.clone(), renderer_raw) {
                Ok(Self {
                    lib,
                    settings,
                    internal: app_internal,
                    monitor,
                    renderer,
                })
            } else {
                lib.appcore().ulDestroyApp(app_internal);
                Err(CreationError::NullReference)
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
        unsafe { self.lib.appcore().ulAppIsRunning(self.internal) }
    }

    /// Get the underlying [`Renderer`] instance.
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
            [App::lib.appcore()] ulAppSetUpdateCallback() {}
    }

    /// Start the main loop.
    pub fn run(&self) {
        unsafe { self.lib.appcore().ulAppRun(self.internal) }
    }

    /// Stop the main loop.
    pub fn quit(&self) {
        unsafe { self.lib.appcore().ulAppQuit(self.internal) }
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
    ///
    /// Returns [`None`] if the window could not be created.
    pub fn create_window(
        &self,
        width: u32,
        height: u32,
        fullscreen: bool,
        window_flags: WindowFlags,
    ) -> Option<Window> {
        unsafe {
            Window::create(
                self.lib.clone(),
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
            self.lib.appcore().ulDestroyApp(self.internal);
        }
    }
}
