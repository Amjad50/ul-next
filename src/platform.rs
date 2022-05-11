//! Platform functions to configure `Ultralight` and provide user-defined
//! implementations for various platform operations.
//!
//! The configurations applied to the platform should be set before creating
//! a [`Renderer`](crate::renderer::Renderer) instance.
use std::{path::Path, sync::Mutex};

use crate::{
    gpu_driver::{self, GpuDriver},
    string::UlString,
};

// static globals for holding Rust implementations of platform structs,
// these will be used on callbacks from the C APIs.
lazy_static::lazy_static! {
    static ref LOGGER: Mutex<Option<Box<dyn Logger + Send>>> = Mutex::new(None);
    static ref CLIPBOARD: Mutex<Option<Box<dyn Clipboard + Send>>> = Mutex::new(None);
    pub(crate) static ref GPUDRIVER: Mutex<Option<Box<dyn GpuDriver + Send>>> = Mutex::new(None);
}

#[derive(Clone, Copy, Debug)]
/// Log levels for the logger. (See [`Logger::log_message`])
pub enum LogLevel {
    /// Info level
    Info = ul_sys::ULLogLevel_kLogLevel_Info as isize,
    /// Warning level
    Warning = ul_sys::ULLogLevel_kLogLevel_Warning as isize,
    /// Error level
    Error = ul_sys::ULLogLevel_kLogLevel_Error as isize,
}

impl TryFrom<u32> for LogLevel {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            ul_sys::ULLogLevel_kLogLevel_Info => Ok(LogLevel::Info),
            ul_sys::ULLogLevel_kLogLevel_Warning => Ok(LogLevel::Warning),
            ul_sys::ULLogLevel_kLogLevel_Error => Ok(LogLevel::Error),
            _ => Err(()),
        }
    }
}

/// This can be used to log debug messages to the console or to a log file.
///
/// This is intended to be implemented by users and defined before creating the Renderer.
///
/// (See [`platform::set_logger`](set_logger))
pub trait Logger {
    /// Invoked when the library wants to print a message to the log.
    fn log_message(&mut self, log_level: LogLevel, message: String);
}

/// This is used for reading and writing data to the platform Clipboard.
///
/// [`App`](crate::app::App) automatically provides a platform-specific implementation of this that cuts, copies,
/// and pastes to the OS clipboard.
///
/// If you are using [`Renderer::create`](crate::renderer::Renderer::create)
/// instead of [`App::new`](crate::app::App::new), you will
/// need to provide your own implementation of this.
/// (See [`platform::set_clipboard`](set_clipboard))
pub trait Clipboard {
    /// Clear the clipboard.
    fn clear(&mut self);

    /// Read plaintext from the clipboard.
    ///
    /// Invoked when the library wants to read from the system's clipboard.
    fn read_plain_text(&mut self) -> Option<String>;

    /// Write plaintext to the clipboard.
    ///
    /// Invoked when the library wants to write to the system's clipboard.
    fn write_plain_text(&mut self, text: &str);
}

platform_set_interface_macro! {
    /// Set a custom Logger implementation.
    ///
    /// This is used to log debug messages to the console or to a log file.
    ///
    /// You should call this before [`App::new`] or [`Renderer::create`].
    ///
    /// [`App::new`] will use the default logger if you never call this.
    ///
    /// If you're [`Renderer::create`] you can still use the
    /// default logger by calling
    /// [`platform::enable_default_logger`](enable_default_logger).
    ///
    /// [`App::new`]: crate::app::App::new
    /// [`Renderer::create`]: crate::renderer::Renderer::create
    pub set_logger<Logger>(logger -> LOGGER) -> ulPlatformSetLogger(ULLogger) {
        // TODO: handle errors
        log_message((ul_log_level: u32, ul_message: ul_sys::ULString)) -> ((log_level: u32, message: String)) {
            let log_level = LogLevel::try_from(ul_log_level).unwrap();
            let message = UlString::copy_raw_to_string(ul_message);
        }
    }
}

platform_set_interface_macro! {
    /// Set a custom Clipboard implementation.
    ///
    /// This should be used if you are using [`Renderer::create`] (which does not provide its own
    /// clipboard implementation).
    ///
    /// The Clipboard trait is used by the library to make calls to the system's native clipboard
    /// (eg, cut, copy, paste).
    ///
    /// You should call this before [`Renderer::create`] or [`App::new`].
    ///
    /// [`App::new`]: crate::app::App::new
    /// [`Renderer::create`]: crate::renderer::Renderer::create
    pub set_clipboard<Clipboard>(clipboard -> CLIPBOARD) -> ulPlatformSetClipboard(ULClipboard) {
        // TODO: handle errors
        clear() -> () {}
        read_plain_text((ul_result: ul_sys::ULString)) -> (() -> result: Option<String>) {
            // no need to preprocess since we're returning a string
        } {
            if let Some(result) = result {
                let result = UlString::from_str(&result);
                ul_sys::ulStringAssignString(ul_result, result.to_ul());
            }
        }
        write_plain_text((ul_text: ul_sys::ULString)) -> ((text: &String)) {
            let text = UlString::copy_raw_to_string(ul_text);
            let text = &text;
        }
    }
}

/// Set a custom GPUDriver implementation.
///
/// This should be used if you have enabled the GPU renderer in the Config and are using
/// [`Renderer`](crate::renderer::Renderer) (which does not provide its own GPUDriver implementation).
///
/// The GpuDriver trait is used by the library to dispatch GPU calls to your native GPU context
/// (eg, D3D11, Metal, OpenGL, Vulkan, etc.) There are reference implementations for this interface
/// in the [`AppCore`](https://github.com/ultralight-ux/AppCore) repo as well
/// as a custom implementation for `glium` in [`glium`](crate::gpu_driver::glium).
///
/// You should call this before [`Renderer::create`](crate::renderer::Renderer::create).
pub fn set_gpu_driver<G: GpuDriver + Send + 'static>(driver: G) {
    gpu_driver::set_gpu_driver(driver)
}

/// Initializes the default logger (writes the log to a file).
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
///
/// You should specify a writable log path to write the log to for example “./ultralight.log”.
pub fn enable_default_logger<P: AsRef<Path>>(log_path: P) {
    unsafe {
        // TODO: handle error
        let log_path = UlString::from_str(log_path.as_ref().to_str().unwrap());
        ul_sys::ulEnableDefaultLogger(log_path.to_ul());
    }
}

/// Initializes the platform font loader and sets it as the current FontLoader.
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
pub fn enable_platform_fontloader() {
    unsafe {
        ul_sys::ulEnablePlatformFontLoader();
    }
}

/// Initializes the platform file system (needed for loading file:/// URLs) and sets it as the current FileSystem.
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
///
/// You can specify a base directory path to resolve relative paths against
pub fn enable_platform_file_system<P: AsRef<Path>>(base_dir: P) {
    unsafe {
        // TODO: handle error
        let base_dir = UlString::from_str(base_dir.as_ref().to_str().unwrap());
        ul_sys::ulEnablePlatformFileSystem(base_dir.to_ul());
    }
}
