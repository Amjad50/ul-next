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

#[macro_use]
pub(crate) mod config_macros;
#[macro_use]
pub(crate) mod callback_macros;

pub mod app;
pub mod bitmap;
pub mod config;
pub mod error;
pub mod event;
pub mod gpu_driver;
pub mod image_source;
pub mod key_code;
pub mod overlay;
pub mod platform;
pub mod rect;
pub mod renderer;
pub(crate) mod string;
pub mod surface;
pub mod view;
pub mod window;

use std::ffi::CStr;

pub use app::App;
pub use config::Config;
pub use gpu_driver::GpuDriver;
pub use overlay::Overlay;
pub use rect::Rect;
pub use renderer::{Renderer, Session};
pub use surface::Surface;
pub use view::View;
pub use window::Window;

#[derive(Clone, Copy, Debug)]
/// The version of the `Ultralight` library.
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

/// Get the current version of the `Ultralight` library.
pub fn version() -> Version {
    unsafe {
        Version {
            major: ul_sys::ulVersionMajor(),
            minor: ul_sys::ulVersionMinor(),
            patch: ul_sys::ulVersionPatch(),
        }
    }
}

/// Get the full WebKit version string
pub fn webkit_version() -> String {
    unsafe {
        let cstr_ptr = ul_sys::ulWebKitVersionString();
        if cstr_ptr.is_null() {
            return String::new();
        }
        let version_cstr = CStr::from_ptr(cstr_ptr);
        version_cstr.to_string_lossy().into_owned()
    }
}
