use std::{path::Path, sync::Mutex};

use crate::string::UlString;

lazy_static::lazy_static! {
    static ref LOGGER: Mutex<Option<Box<dyn Logger + Send>>> = Mutex::new(None);
    static ref CLIPBOARD: Mutex<Option<Box<dyn Clipboard + Send>>> = Mutex::new(None);
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info = ul_sys::ULLogLevel_kLogLevel_Info as isize,
    Warning = ul_sys::ULLogLevel_kLogLevel_Warning as isize,
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

pub trait Logger {
    /// Invoked when the library wants to print a message to the log.
    fn log(&mut self, log_level: LogLevel, message: String);
}

pub trait Clipboard {
    /// Invoked when the library wants to clear the system's clipboard.
    fn clear(&mut self);
    /// Invoked when the library wants to read from the system's clipboard.
    fn read_plain_text(&mut self) -> Option<String>;
    /// Invoked when the library wants to write to the system's clipboard.
    fn write_plain_text(&mut self, text: &str);
}

pub struct Platform;

impl Platform {
    /// Set a custom Logger implementation.
    ///
    /// This is used to log debug messages to the console or to a log file.
    ///
    /// You should call this before [`App::new`] or [`Renderer::create`]
    ///
    /// [`App::new`] will use the default logger if you never call this.
    ///
    /// If you're [`Renderer::create`] you can still use the
    /// default logger by calling [`Platform::enable_default_logger`].
    pub fn set_logger<L: Logger + Send + 'static>(logger: L) {
        let logger = Box::new(logger);
        *LOGGER.lock().unwrap() = Some(logger);

        // TODO: handle errors
        unsafe extern "C" fn trampoline(log_level: u32, message: ul_sys::ULString) {
            let log_level = LogLevel::try_from(log_level).unwrap();
            let message = UlString::copy_raw_to_string(message);
            let mut logger = LOGGER.lock().unwrap();
            // the logger must always be `Some` at this point.
            logger.as_mut().unwrap().log(log_level, message);
        }

        let ul_logger = ul_sys::ULLogger {
            log_message: Some(trampoline),
        };

        unsafe {
            ul_sys::ulPlatformSetLogger(ul_logger);
        }
    }

    /// Set a custom Clipboard implementation.
    ///
    /// This should be used if you are using [`Renderer::create`] (which does not provide its own
    /// clipboard implementation).
    ///
    /// The Clipboard trait is used by the library to make calls to the system's native clipboard
    /// (eg, cut, copy, paste).
    ///
    /// You should call this before [`Renderer::create`] or [`App::new`].
    pub fn set_clipboard<C: Clipboard + Send + 'static>(clipboard: C) {
        let clipboard = Box::new(clipboard);
        *CLIPBOARD.lock().unwrap() = Some(clipboard);

        // TODO: handle errors
        unsafe extern "C" fn trampoline_clear() {
            let mut clipboard = CLIPBOARD.lock().unwrap();
            // the clipboard must always be `Some` at this point.
            clipboard.as_mut().unwrap().clear();
        }

        unsafe extern "C" fn trampoline_read_plain_text(ul_result: ul_sys::ULString) {
            let mut clipboard = CLIPBOARD.lock().unwrap();
            // the clipboard must always be `Some` at this point.
            let result = clipboard.as_mut().unwrap().read_plain_text();
            if let Some(result) = result {
                let result = UlString::from_str(&result);
                ul_sys::ulStringAssignString(ul_result, result.to_ul());
            }
            // no need to write if there is no data
        }

        unsafe extern "C" fn trampoline_write_plain_text(ul_text: ul_sys::ULString) {
            let text = UlString::copy_raw_to_string(ul_text);
            let mut clipboard = CLIPBOARD.lock().unwrap();
            // the clipboard must always be `Some` at this point.
            clipboard.as_mut().unwrap().write_plain_text(&text);
        }

        let ul_clipboard = ul_sys::ULClipboard {
            clear: Some(trampoline_clear),
            read_plain_text: Some(trampoline_read_plain_text),
            write_plain_text: Some(trampoline_write_plain_text),
        };

        unsafe {
            ul_sys::ulPlatformSetClipboard(ul_clipboard);
        }
    }

    /// This is only needed if you are not calling [`App::new`]
    ///
    /// Initializes the default logger (writes the log to a file).
    ///
    /// You should specify a writable log path to write the log to for example “./ultralight.log”.
    pub fn enable_default_logger<P: AsRef<Path>>(log_path: P) {
        unsafe {
            // TODO: handle error
            let log_path = UlString::from_str(log_path.as_ref().to_str().unwrap());
            ul_sys::ulEnableDefaultLogger(log_path.to_ul());
        }
    }

    /// This is only needed if you are not calling [`App::new`]
    ///
    /// Initializes the platform font loader and sets it as the current FontLoader.
    pub fn enable_platform_fontloader() {
        unsafe {
            ul_sys::ulEnablePlatformFontLoader();
        }
    }

    /// This is only needed if you are not calling [`App::new`]
    ///
    /// Initializes the platform file system (needed for loading file:/// URLs) and sets it as the current FileSystem.
    ///
    /// You can specify a base directory path to resolve relative paths against
    pub fn enable_platform_file_system<P: AsRef<Path>>(base_dir: P) {
        unsafe {
            // TODO: handle error
            let base_dir = UlString::from_str(base_dir.as_ref().to_str().unwrap());
            ul_sys::ulEnablePlatformFileSystem(base_dir.to_ul());
        }
    }
}
