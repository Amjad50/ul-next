//! A safe Rust wrapper for [`Ultralight`](https://ultralig.ht/) library.
//!
//! `Ultralight` is a library for rendering web content using the GPU, it allows
//! easy integration into games and other applications.
//!
//! There are two options to use the library:
//! - Using the [`App`] struct, which is a managed application that allows
//!   to create [`Window`]s that can contain multiple [`Overlay`]s, you can
//!   control the position and size of the [`Overlay`]s, and the inner [`View`]s.
//! - The other option is using the [`Renderer`] directly, in that case, if you
//!   want to have GPU rendering in your application you need to supply a custom
//!   [`GpuDriver`] in [`platform::set_gpu_driver`].
//!
//! This library also contain a custom [`glium`](crate::gpu_driver::glium)
//! [`GpuDriver`] implementation that can be used for easier integration.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "linked", feature = "loaded")))]
compile_error!(
    "At least one of the features `linked`, `appcore_linked` or `loaded` must be enabled."
);

#[macro_use]
pub(crate) mod config_macros;
#[macro_use]
pub(crate) mod callback_macros;

#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub mod app;
pub mod bitmap;
pub mod config;
pub mod error;
pub mod event;
pub mod gpu_driver;
pub mod image_source;
pub mod key_code;
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub mod overlay;
pub mod platform;
pub mod rect;
pub mod renderer;
pub(crate) mod string;
pub mod surface;
pub mod view;
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub mod window;

pub mod javascript;

use std::{ffi::CStr, sync::Arc};

#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub use app::App;
pub use config::Config;
pub use gpu_driver::GpuDriver;
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub use overlay::Overlay;
pub use rect::Rect;
pub use renderer::{Renderer, Session};
pub use surface::Surface;
pub use view::View;
#[cfg(any(feature = "appcore_linked", feature = "loaded"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "appcore_linked", feature = "loaded"))))]
pub use window::Window;

use ul_sys::library::Library as LibrarySys;

#[derive(Clone, Copy, Debug)]
/// The version of the `Ultralight` library.
///
/// Use the [`Library::version`] method to get the current version of the library.
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Convert the version into a string in the format `MAJOR.MINOR.PATCH`.
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A handle to the `Ultralight` library.
#[derive(Clone)]
pub struct Library {
    lib: LibrarySys,
}

impl Library {
    /// Creates a new `Library` instance with linked Ultralight and AppCore (if enabled `appcore_linked` feature) functions.
    #[cfg(any(feature = "linked", feature = "appcore_linked"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "linked", feature = "appcore_linked"))))]
    pub fn linked() -> Arc<Library> {
        Arc::new(Library {
            lib: LibrarySys::linked(),
        })
    }

    /// Loads the Ultralight library for the current platform.
    ///
    /// The library must be installed in the system library path, or loadable by name.
    ///
    /// All the other related libraries (`UltralightCore` and `WebCore`) must be loadable as well.
    ///
    /// This is preferred over [`linked()`][Library::linked] when the application
    /// wants to gracefully handle the absence of the library, or handle loading it dynamically.
    ///
    /// This doesn't come with `AppCore` functions, use [`load_with_appcore()`][Library::load_with_appcore]
    /// if you need `AppCore` functions.
    ///
    /// # Safety
    ///
    /// `dlopen` native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.new>] and
    ///     [`Library::get()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.get>] apply here.
    ///
    /// No function loaded directly or indirectly from this [`Library`]
    /// may be called after it is [dropped][drop()].
    #[cfg(feature = "loaded")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
    pub unsafe fn load() -> Result<Arc<Library>, ul_sys::library::LoadingError> {
        Ok(Arc::new(Library {
            lib: LibrarySys::load()?,
        }))
    }

    /// Loads the AppCore and Ultralight libraries for the current platform.
    ///
    /// The libraries must be installed in the system library path, or loadable by name.
    ///
    /// All the other related libraries (`Ultralight`, `UltralightCore` and `WebCore`) must be loadable as well.
    ///
    /// This comes with all functions, `Ultralight` and `AppCore`.
    /// If you are handling your own rendering and windowing,
    /// you may want to use [`load()`][Library::load] only instead
    /// which doesn't come with `AppCore` functions.
    ///
    /// # Safety
    ///
    /// `dlopen` native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.new>] and
    ///     [`Library::get()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.get>] apply here.
    ///
    /// No function loaded directly or indirectly from this [`Library`]
    /// may be called after it is [dropped][drop()].
    #[cfg(feature = "loaded")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
    pub unsafe fn load_with_appcore() -> Result<Arc<Library>, ul_sys::library::LoadingError> {
        Ok(Arc::new(Library {
            lib: LibrarySys::load_with_appcore()?,
        }))
    }

    /// Loads the Ultralight library from the given path/name of the library.
    ///
    /// All the other related libraries (`UltralightCore` and `WebCore`) must be loadable as well.
    ///
    /// # Safety
    ///
    /// `dlopen` native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.new>] and
    ///     [`Library::get()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.get>] apply here.
    ///
    /// No function loaded directly or indirectly from this [`Library`]
    /// may be called after it is [dropped][drop()].
    #[cfg(feature = "loaded")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
    pub unsafe fn load_from<P: AsRef<::std::ffi::OsStr>>(
        ultralight_path: P,
    ) -> Result<Arc<Library>, ul_sys::library::LoadingError> {
        Ok(Arc::new(Library {
            lib: LibrarySys::load_from(ultralight_path.as_ref())?,
        }))
    }

    /// Loads the AppCore and Ultralight libraries from the given path/name of the library.
    ///
    /// All the other related libraries (`Ultralight`, `UltralightCore` and `WebCore`) must be loadable as well.
    ///
    /// This will load all functions, `Ultralight` and `AppCore`.
    /// If you are handling your own rendering and windowing,
    /// you may want to use [`load_from()`][Library::load_from] only instead
    /// which won't load `AppCore` functions.
    ///
    /// # Safety
    ///
    /// `dlopen` native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.new>] and
    ///     [`Library::get()`][<https://docs.rs/libloading/latest/libloading/struct.Library.html#method.get>] apply here.
    ///
    /// No function loaded directly or indirectly from this [`Library`]
    /// may be called after it is [dropped][drop()].
    #[cfg(feature = "loaded")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
    pub unsafe fn load_from_appcore<P>(
        appcore_path: P,
    ) -> Result<Arc<Library>, ul_sys::library::LoadingError>
    where
        P: AsRef<::std::ffi::OsStr>,
    {
        Ok(Arc::new(Library {
            lib: LibrarySys::load_from_appcore(appcore_path.as_ref())?,
        }))
    }
}

impl Library {
    pub(crate) fn ultralight(&self) -> &ul_sys::library::Ultralight {
        self.lib.ultralight()
    }

    #[cfg(any(feature = "appcore_linked", feature = "loaded"))]
    pub(crate) fn appcore(&self) -> &ul_sys::library::AppCore {
        self.lib.appcore()
    }

    /// Get the current version of the `Ultralight` library.
    pub fn version(&self) -> Version {
        unsafe {
            Version {
                major: self.lib.ultralight().ulVersionMajor(),
                minor: self.lib.ultralight().ulVersionMinor(),
                patch: self.lib.ultralight().ulVersionPatch(),
            }
        }
    }

    /// Get the full WebKit version string
    pub fn webkit_version(&self) -> String {
        unsafe {
            let cstr_ptr = self.lib.ultralight().ulWebKitVersionString();
            if cstr_ptr.is_null() {
                return String::new();
            }
            let version_cstr = CStr::from_ptr(cstr_ptr);
            version_cstr.to_string_lossy().into_owned()
        }
    }
}
