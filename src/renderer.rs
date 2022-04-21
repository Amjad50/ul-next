use crate::{
    config::Config,
    session::Session,
    string::UlString,
    view::{View, ViewConfig},
};

pub struct Renderer {
    internal: ul_sys::ULRenderer,

    need_to_destroy: bool,
}

impl Renderer {
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULRenderer) -> Self {
        Self {
            internal: raw,
            need_to_destroy: false,
        }
    }

    pub fn create(config: Config) -> Self {
        let internal = unsafe { ul_sys::ulCreateRenderer(config.to_ul()) };
        Self {
            internal,
            need_to_destroy: true,
        }
    }
}

impl Renderer {
    pub fn update(&self) {
        unsafe { ul_sys::ulUpdate(self.internal) };
    }

    pub fn render(&self) {
        unsafe { ul_sys::ulRender(self.internal) };
    }

    pub fn purge_memory(&self) {
        unsafe { ul_sys::ulPurgeMemory(self.internal) };
    }

    pub fn log_memory_usage(&self) {
        unsafe { ul_sys::ulLogMemoryUsage(self.internal) };
    }

    pub fn create_session(&self, is_persistent: bool, name: &str) -> Session {
        unsafe { Session::create(self.internal, is_persistent, name) }
    }

    pub fn default_session(&self) -> Session {
        unsafe { Session::from_raw(ul_sys::ulDefaultSession(self.internal)) }
    }

    pub fn create_view(
        &self,
        width: u32,
        height: u32,
        view_config: &ViewConfig,
        session: Option<Session>,
    ) -> View {
        unsafe { View::create(self.internal, width, height, &view_config, session) }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                ul_sys::ulDestroyRenderer(self.internal);
            }
        }
    }
}
