#[macro_use]
pub(crate) mod config_macros;
#[macro_use]
pub(crate) mod callback_macros;

pub mod app;
pub mod bitmap;
pub mod config;
pub mod event;
pub mod gpu_driver;
pub mod overlay;
pub mod platform;
pub mod rect;
pub mod render_target;
pub mod renderer;
pub mod session;
pub mod string;
pub mod surface;
pub mod view;
pub mod window;

#[derive(Clone, Copy, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub fn version() -> Version {
    unsafe {
        Version {
            major: ul_sys::ulVersionMajor(),
            minor: ul_sys::ulVersionMinor(),
            patch: ul_sys::ulVersionPatch(),
        }
    }
}
