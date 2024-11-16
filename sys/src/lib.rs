#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod defines;

#[cfg(feature = "linked")]
#[cfg_attr(docsrs, doc(cfg(feature = "linked")))]
pub mod linked {
    #[cfg(feature = "appcore_linked")]
    #[cfg_attr(docsrs, doc(cfg(feature = "appcore_linked")))]
    mod appcore;
    mod ultralight;

    #[cfg(feature = "appcore_linked")]
    pub use appcore::*;
    pub use ultralight::*;
}

pub use defines::*;

#[allow(private_bounds)]
pub mod library {
    #[allow(clippy::missing_safety_doc)]
    mod appcore;
    #[allow(clippy::missing_safety_doc)]
    mod ultralight;

    pub use appcore::*;
    pub use ultralight::*;

    #[cfg(feature = "loaded")]
    pub use libloading::Error as LoadingError;

    /// structure that holds `Ultralight` and optionally `AppCore` functions.
    #[derive(Clone)]
    pub struct Library {
        ultralight_lib: Ultralight,
        appcore_lib: Option<AppCore>,
    }

    impl Library {
        /// Creates a new `Library` instance with linked Ultralight and AppCore (if enabled `appcore_linked` feature) functions.
        #[cfg(any(feature = "linked", feature = "appcore_linked"))]
        #[cfg_attr(docsrs, doc(cfg(any(feature = "linked", feature = "appcore_linked"))))]
        pub fn linked() -> Library {
            Library {
                ultralight_lib: Ultralight::linked(),
                #[cfg(feature = "appcore_linked")]
                appcore_lib: Some(AppCore::linked()),
                #[cfg(not(feature = "appcore_linked"))]
                appcore_lib: None,
            }
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
        /// for [`Library::new()`][libloading::Library::new] and
        ///     [`Library::get()`][libloading::Library::get] apply here.
        ///
        /// No function loaded directly or indirectly from this [`Library`]
        /// may be called after it is [dropped][drop()].
        #[cfg(feature = "loaded")]
        #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
        pub unsafe fn load() -> Result<Library, libloading::Error> {
            #[cfg(windows)]
            const LIB_PATH: &str = "Ultralight.dll";

            #[cfg(all(unix, not(target_os = "macos")))]
            const LIB_PATH: &str = "libUltralight.so";

            #[cfg(target_os = "macos")]
            const LIB_PATH: &str = "libUltralight.dylib";

            Self::load_from(LIB_PATH)
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
        /// for [`Library::new()`][libloading::Library::new] and
        ///     [`Library::get()`][libloading::Library::get] apply here.
        ///
        /// No function loaded directly or indirectly from this [`Library`]
        /// may be called after it is [dropped][drop()].
        #[cfg(feature = "loaded")]
        #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
        pub unsafe fn load_with_appcore() -> Result<Library, libloading::Error> {
            #[cfg(windows)]
            const LIB_PATH: &str = "AppCore.dll";

            #[cfg(all(unix, not(target_os = "macos")))]
            const LIB_PATH: &str = "libAppCore.so";

            #[cfg(target_os = "macos")]
            const LIB_PATH: &str = "libAppCore.dylib";

            Self::load_from_appcore(LIB_PATH)
        }

        /// Loads the Ultralight library from the given path/name of the library.
        ///
        /// All the other related libraries (`UltralightCore` and `WebCore`) must be loadable as well.
        ///
        /// # Safety
        ///
        /// `dlopen` native libraries is inherently unsafe. The safety guidelines
        /// for [`Library::new()`][libloading::Library::new] and
        ///     [`Library::get()`][libloading::Library::get] apply here.
        ///
        /// No function loaded directly or indirectly from this [`Library`]
        /// may be called after it is [dropped][drop()].
        #[cfg(feature = "loaded")]
        #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
        pub unsafe fn load_from<P: AsRef<::std::ffi::OsStr>>(
            ultralight_path: P,
        ) -> Result<Library, libloading::Error> {
            Ok(Library {
                ultralight_lib: Ultralight::load_from(ultralight_path.as_ref())?,
                appcore_lib: None,
            })
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
        /// for [`Library::new()`][libloading::Library::new] and
        ///     [`Library::get()`][libloading::Library::get] apply here.
        ///
        /// No function loaded directly or indirectly from this [`Library`]
        /// may be called after it is [dropped][drop()].
        #[cfg(feature = "loaded")]
        #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
        pub unsafe fn load_from_appcore<P>(appcore_path: P) -> Result<Library, libloading::Error>
        where
            P: AsRef<::std::ffi::OsStr>,
        {
            Ok(Library {
                ultralight_lib: Ultralight::load_from(appcore_path.as_ref())?,
                appcore_lib: Some(AppCore::load_from(appcore_path.as_ref())?),
            })
        }
    }

    impl Library {
        /// Returns a reference to the Ultralight library.
        pub fn ultralight(&self) -> &Ultralight {
            &self.ultralight_lib
        }

        /// Returns a reference to the AppCore library.
        ///
        /// # Panics
        /// Panics if `AppCore` is not enabled.
        pub fn appcore(&self) -> &AppCore {
            self.appcore_lib.as_ref().expect("AppCore is not enabled")
        }

        /// Returns a reference to the AppCore library if enabled.
        pub fn try_appcore(&self) -> Option<&AppCore> {
            self.appcore_lib.as_ref()
        }
    }
}
