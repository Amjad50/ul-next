use std::{path::Path, sync::Mutex};

use crate::string::UlString;

lazy_static::lazy_static! {
    static ref LOGGER: Mutex<Option<Box<dyn Logger + Send>>> = Mutex::new(None);
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
    fn log(&mut self, log_level: LogLevel, message: String);
}

pub struct Platform;

impl Platform {
    pub fn set_logger<L: Logger + Send + 'static>(logger: L) {
        let logger = Box::new(logger);
        *LOGGER.lock().unwrap() = Some(logger);

        // handle errors
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
