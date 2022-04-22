use crate::{
    config::Config,
    session::Session,
    view::{View, ViewConfig},
};

pub struct Renderer {
    internal: ul_sys::ULRenderer,

    need_to_destroy: bool,

    default_session: Session,
}

impl Renderer {
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULRenderer) -> Self {
        let default_session = Session::from_raw(ul_sys::ulDefaultSession(raw));

        Self {
            internal: raw,
            need_to_destroy: false,
            default_session,
        }
    }

    pub fn create(config: Config) -> Self {
        let internal = unsafe { ul_sys::ulCreateRenderer(config.to_ul()) };
        let default_session = unsafe { Session::from_raw(ul_sys::ulDefaultSession(internal)) };

        Self {
            internal,
            need_to_destroy: true,
            default_session,
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

    pub fn default_session(&self) -> &Session {
        &self.default_session
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
