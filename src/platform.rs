//! Platform functions to configure `Ultralight` and provide user-defined
//! implementations for various platform operations.
//!
//! The configurations applied to the platform should be set before creating
//! a [`Renderer`](crate::renderer::Renderer) instance.
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[allow(unused_imports)]
use crate::error::CreationError;

use crate::{
    gpu_driver::{self, GpuDriver},
    string::UlString,
    Library,
};

// static globals for holding Rust implementations of platform structs,
// these will be used on callbacks from the C APIs.
lazy_static::lazy_static! {
    static ref LOGGER: InternalPlatform<Box<dyn Logger + Send>> = InternalPlatform::new();
    static ref CLIPBOARD: InternalPlatform<Box<dyn Clipboard + Send>> = InternalPlatform::new();
    static ref FILESYSTEM: InternalPlatform<Box<dyn FileSystem + Send>> = InternalPlatform::new();
    static ref FONTLOADER: InternalPlatform<Box<dyn FontLoader + Send>> = InternalPlatform::new();
    pub(crate) static ref GPUDRIVER: InternalPlatform<Box<dyn GpuDriver + Send>> = InternalPlatform::new();
}

pub(crate) struct InternalPlatform<T> {
    pub(crate) lib: Mutex<Option<Arc<Library>>>,
    pub(crate) obj: Mutex<Option<T>>,
}

impl<T> InternalPlatform<T> {
    pub(crate) const fn new() -> Self {
        Self {
            lib: Mutex::new(None),
            obj: Mutex::new(None),
        }
    }
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

/// This is used for loading File URLs (eg, <file:///page.html>).
///
/// You can provide the library with your own FileSystem implementation so that file assets are
/// loaded from your own pipeline (useful if you would like to encrypt/compress your file assets or
/// ship it in a custom format).
///
/// AppCore automatically provides a platform-specific implementation of this that loads files from
/// a local directory when you call [`App::new`]
///
/// If you are using [`Renderer::create`] instead, you will need to provide your own implementation
/// via [`platform::set_filesystem`].
///
/// To provide your own custom FileSystem implementation, you should implement
/// this trait, and then pass an instance of your struct to
/// [`platform::set_filesystem`] before calling [`Renderer::create`] or [`App::new`].
///
/// [`App::new`]: crate::app::App::new
/// [`Renderer::create`]: crate::renderer::Renderer::create
/// [`platform::set_filesystem`]: set_filesystem
pub trait FileSystem {
    /// Check if file path exists, return true if exists.
    fn file_exists(&mut self, path: &str) -> bool;

    /// Get the mime-type of the file (eg "text/html").
    ///
    /// This is usually determined by analyzing the file extension.
    ///
    /// If a mime-type cannot be determined, this should return "application/unknown".
    fn get_file_mime_type(&mut self, path: &str) -> String;

    /// Get the charset / encoding of the file (eg "utf-8", "iso-8859-1").
    ///
    /// This is only applicable for text-based files (eg, "text/html", "text/plain")
    /// and is usually determined by analyzing the contents of the file.
    ///
    /// If a charset cannot be determined, a safe default to return is "utf-8".
    fn get_file_charset(&mut self, path: &str) -> String;

    /// Open file for reading and map it to a Buffer.
    ///
    /// If the file was unable to be opened, you should return `None`.
    fn open_file(&mut self, path: &str) -> Option<Vec<u8>>;
}

/// Represents a font file, either on-disk path or in-memory file contents.
pub struct FontFile {
    lib: Arc<Library>,
    internal: ul_sys::ULFontFile,
}

impl FontFile {
    // TODO: support buffers
    // pub fn from_buffer(...) -> Option<Self> {
    //}

    /// Create a font file from an on-disk file path.
    ///
    /// The file path should already exist.
    pub fn from_path<P: AsRef<Path>>(lib: Arc<Library>, path: P) -> Option<Self> {
        unsafe {
            let path = UlString::from_str(lib.clone(), path.as_ref().to_str().unwrap()).unwrap();

            let internal = lib.ultralight().ulFontFileCreateFromFilePath(path.to_ul());
            if internal.is_null() {
                return None;
            }
            Some(FontFile { lib, internal })
        }
    }

    #[allow(dead_code)]
    pub(crate) fn to_ul(&self) -> ul_sys::ULFontFile {
        self.internal
    }
}

impl Drop for FontFile {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().ulDestroyFontFile(self.internal);
        }
    }
}

/// User-defined font loader interface.
///
/// The library uses this to load all system fonts.
///
/// Every operating system has its own library of installed system fonts. The FontLoader interface
/// is used to lookup these fonts and fetch the actual font data (raw TTF/OTF file data) for a given
/// given font description.
///
/// ## Usage
///
/// To provide your own custom FontLoader implementation, you should implement this trait, and then
/// pass an instance of your struct to [`platform::set_fontloader`]. before calling [`Renderer::create`]
/// or [`App::new`].
///
/// ## Note
///
/// > There is a bug in Ultralight right now, and we can't use custom Fontloader for now. So you
/// > can ignore this for now.
///
/// AppCore uses a default OS-specific FontLoader implementation when you call [`App::new`].
///
/// If you are using [`Renderer::create`], you can still use AppCore's implementation by calling
/// [`platform::enable_platform_fontloader`][enable_platform_fontloader].
///
///
/// [`App::new`]: crate::app::App::new
/// [`Renderer::create`]: crate::renderer::Renderer::create
pub trait FontLoader {
    /// Fallback font family name. Will be used if all other fonts fail to load.
    ///
    /// This font should be guaranteed to exist (eg, ULFontLoader::load should not fail when
    /// when passed this font family name).
    ///
    /// Return a font family name.
    fn get_fallback_font(&mut self) -> String;

    /// Fallback font family name that can render the specified characters. This is mainly used to
    /// support CJK (Chinese, Japanese, Korean) text display
    ///
    /// # Arguments
    /// * `characters` - One or more UTF-16 characters. This is almost always a single character
    /// * `weight` - Font weight
    /// * `italic` - Whether or not the font should be italic
    ///
    /// Should return a font family name that can render the text
    fn get_fallback_font_for_characters(
        &mut self,
        characters: &str,
        weight: i32,
        italic: bool,
    ) -> String;

    /// Get the actual font file data (TTF/OTF) for a given font description.
    ///
    /// # Arguments
    /// * `family` - Font family name
    /// * `weight` - Font weight
    /// * `italic` - Whether or not the font should be italic
    ///
    /// Should return a [`FontFile`] that contains the font data. You can return `None`
    /// here and the loader will fallback to another font.
    fn load(&mut self, family: &str, weight: i32, italic: bool) -> Option<FontFile>;
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
    pub set_logger<Logger>(lib, logger -> LOGGER) -> ulPlatformSetLogger(ULLogger) {
        // TODO: handle errors
        log_message((ul_log_level: u32, ul_message: ul_sys::ULString)) -> ((log_level: u32, message: String)) {
            let log_level = LogLevel::try_from(ul_log_level).unwrap();
            let message = UlString::copy_raw_to_string(&lib, ul_message).unwrap();
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
    pub set_clipboard<Clipboard>(lib, clipboard -> CLIPBOARD) -> ulPlatformSetClipboard(ULClipboard) {
        // TODO: handle errors
        clear() -> () {}
        read_plain_text((ul_result: ul_sys::ULString)) -> (() -> result: Option<String>) {
            // no need to preprocess since we're returning a string
        } {
            if let Some(result) = result {
                let result = UlString::from_str(lib.clone(), &result).unwrap();
                lib.ultralight().ulStringAssignString(ul_result, result.to_ul());
            }
        }
        write_plain_text((ul_text: ul_sys::ULString)) -> ((text: &String)) {
            let text = UlString::copy_raw_to_string(&lib, ul_text).unwrap();
            let text = &text;
        }
    }
}

platform_set_interface_macro! {
    /// Set a custom FileSystem implementation.
    ///
    /// The library uses this to load all file URLs (eg, <file:///page.html>).
    ///
    /// You can provide the library with your own FileSystem implementation
    /// so that file assets are loaded from your own pipeline.
    ///
    /// You should call this before [`Renderer::create`] or [`App::new`].
    ///
    /// Note: [`App::new`] will use the default platform file system if you never call this.
    ///
    /// If you're not using [`App::new`], (eg, using [`Renderer::create`]) you
    /// can still use the default platform file system by calling
    /// [`platform::enable_platform_filesystem`](enable_platform_filesystem).
    ///
    /// [`App::new`]: crate::app::App::new
    /// [`Renderer::create`]: crate::renderer::Renderer::create
    pub set_filesystem<FileSystem>(lib, filesystem -> FILESYSTEM) -> ulPlatformSetFileSystem(ULFileSystem) {
        // TODO: handle errors
        file_exists((ul_path: ul_sys::ULString) -> bool) -> ((path: &str)) {
            let path = UlString::copy_raw_to_string(&lib, ul_path).unwrap();
            let path = &path;
        }
        get_file_mime_type((ul_path: ul_sys::ULString) -> ul_sys::ULString) -> ((path: &str) -> result: String) {
            let path = UlString::copy_raw_to_string(&lib, ul_path).unwrap();
            let path = &path;
        } {
            UlString::from_str_unmanaged(&lib, &result).unwrap()
        }
        get_file_charset((ul_path: ul_sys::ULString) -> ul_sys::ULString) -> ((path: &str) -> result: String) {
            let path = UlString::copy_raw_to_string(&lib, ul_path).unwrap();
            let path = &path;
        } {
            UlString::from_str_unmanaged(&lib, &result).unwrap()
        }
        open_file((ul_path: ul_sys::ULString) -> ul_sys::ULBuffer) -> ((path: &str) -> result: Option<Vec<u8>>) {
            let path = UlString::copy_raw_to_string(&lib, ul_path).unwrap();
            let path = &path;
        } {
            if let Some(result) = result {
                lib.ultralight().ulCreateBufferFromCopy(result.as_ptr() as _, result.len())
            } else{
                std::ptr::null_mut()
            }
        }
    }
}

// TODO: for some reason, `ulPlatformSetFontLoader` is found in the headers, but not yet the binaries
// platform_set_interface_macro! {
//     /// Set a custom FontLoader implementation.
//     ///
//     /// The library uses this to load all system fonts.
//     ///
//     /// Every operating system has its own library of installed system fonts. The FontLoader interface
//     /// is used to lookup these fonts and fetch the actual font data (raw TTF/OTF file data) for a given
//     /// given font description.
//     ///
//     /// You should call this before [`Renderer::create`] or [`App::new`].
//     ///
//     /// Note: [`App::new`] will use the default platform font loader if you never call this.
//     ///
//     /// [`App::new`]: crate::app::App::new
//     /// [`Renderer::create`]: crate::renderer::Renderer::create
//     pub set_fontloader<FontLoader>(font_loader -> FONTLOADER) -> ulPlatformSetFontLoader(ULFontLoader) {
//         // TODO: handle errors
//         get_fallback_font(() -> ul_sys::ULString) -> (() -> result: String) {
//             // no need to preprocess since we're returning a string
//         } {
//             UlString::from_str_unmanaged(&result).unwrap()
//         }

//         get_fallback_font_for_characters(
//             (ul_characters: ul_sys::ULString, weight: i32, italic: bool) -> ul_sys::ULString) ->
//             ((characters: &str, weight: i32, italic: bool) -> result: String)
//         {
//             let characters = UlString::copy_raw_to_string(ul_characters).unwrap();
//             let characters = &characters;
//         } {
//             UlString::from_str_unmanaged(&result).unwrap()
//         }

//         load((ul_family: ul_sys::ULString, weight: i32, italic: bool) -> ul_sys::ULFontFile) ->
//             ((family: &str, weight: i32, italic: bool) -> result: Option<FontFile>)
//         {
//             let family = UlString::copy_raw_to_string(ul_family).unwrap();
//             let family = &family;
//         } {
//             if let Some(result) = result {
//                 let r = result.to_ul();
//                 // Assuming Ultralight will take ownership of this
//                 core::mem::forget(result);
//                 r
//             } else {
//                 std::ptr::null_mut()
//             }
//         }
//     }
// }

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
pub fn set_gpu_driver<G: GpuDriver + Send + 'static>(lib: Arc<Library>, driver: G) {
    gpu_driver::set_gpu_driver(lib, driver)
}

/// Initializes the default logger (writes the log to a file).
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
///
/// You should specify a writable log path to write the log to for example “./ultralight.log”.
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub fn enable_default_logger<P: AsRef<Path>>(
    lib: Arc<Library>,
    log_path: P,
) -> Result<(), CreationError> {
    unsafe {
        // TODO: handle error
        let log_path = UlString::from_str(lib.clone(), log_path.as_ref().to_str().unwrap())?;
        lib.appcore().ulEnableDefaultLogger(log_path.to_ul());
    }
    Ok(())
}

/// Initializes the platform font loader and sets it as the current FontLoader.
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub fn enable_platform_fontloader(lib: Arc<Library>) {
    unsafe {
        lib.appcore().ulEnablePlatformFontLoader();
    }
}

/// Initializes the platform file system (needed for loading file:/// URLs) and sets it as the current FileSystem.
///
/// This is only needed if you are not calling [`App::new`](crate::app::App::new)
///
/// You can specify a base directory path to resolve relative paths against
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub fn enable_platform_filesystem<P: AsRef<Path>>(
    lib: Arc<Library>,
    base_dir: P,
) -> Result<(), CreationError> {
    unsafe {
        // TODO: handle error
        let base_dir = UlString::from_str(lib.clone(), base_dir.as_ref().to_str().unwrap())?;
        lib.appcore().ulEnablePlatformFileSystem(base_dir.to_ul());
    }
    Ok(())
}
