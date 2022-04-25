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

    /// Create the Ultralight Renderer directly.
    ///
    /// Unlike [`App::new`], this does not use any native windows for drawing and allows you to manage
    /// your own runloop and painting. This method is recommended for those wishing to integrate the
    /// library into a game.
    ///
    /// This singleton manages the lifetime of all Views and coordinates all painting, rendering,
    /// network requests, and event dispatch.
    ///
    /// You should only call this once per process lifetime.
    ///
    /// You shoud set up your platform handlers (eg,
    /// [`Platform::set_logger`]/[`Platform::enable_default_logger`],
    /// [`Platform::enable_platform_file_system`], etc.) before calling this.
    ///
    /// You will also need to define a font loader before calling this -- as of this writing (v1.2) the
    /// only way to do this is [`Platform::enable_platform_fontloader`]
    ///
    /// @NOTE:  You should not call this if you are using [`App::new`], it creates its own renderer and
    ///         provides default implementations for various platform handlers automatically.
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
