//! The `Renderer` manages all [`View`]s  and coordinates painting,
//! network requests, and event dispatch
//!
//! [`Renderer`] should be used when you want to access GPU internals and textures
//! to integerate with your own rendering pipeline.
//!
//! Before creating a renderer [`Renderer::create`] you must supply a custom
//! [`GpuDriver`](crate::gpu_driver::GpuDriver) in
//! [`platform::set_gpu_driver`](crate::platform::set_gpu_driver).
use crate::{
    config::Config,
    string::UlString,
    view::{View, ViewConfig},
};

/// A Session stores local data such as cookies, local storage, and application
/// cache for one or more [`View`](crate::view::View)s.
/// (See [`Renderer::create_session`](crate::renderer::Renderer::create_session))
pub struct Session {
    internal: ul_sys::ULSession,
    need_to_destroy: bool,

    is_persistent: bool,
    name: String,
    id: u64,
    disk_path: String,
}

impl Session {
    /// Internal function helper to create a session.
    /// (See [`Renderer::create_session`](crate::renderer::Renderer::create_session))
    pub(crate) unsafe fn create(
        renderer: ul_sys::ULRenderer,
        is_persistent: bool,
        name: &str,
    ) -> Self {
        let ul_string_name = UlString::from_str(name);
        let internal = ul_sys::ulCreateSession(renderer, is_persistent, ul_string_name.to_ul());

        let id = ul_sys::ulSessionGetId(internal);
        let disk_path = UlString::copy_raw_to_string(ul_sys::ulSessionGetDiskPath(internal));

        Self {
            internal,
            need_to_destroy: true,

            is_persistent,
            name: name.to_string(),
            id,
            disk_path,
        }
    }

    /// Helper internal function to allow getting a reference to a managed
    /// session.
    pub(crate) unsafe fn from_raw(raw: ul_sys::ULSession) -> Self {
        let id = ul_sys::ulSessionGetId(raw);
        let disk_path = UlString::copy_raw_to_string(ul_sys::ulSessionGetDiskPath(raw));
        let name = UlString::copy_raw_to_string(ul_sys::ulSessionGetName(raw));
        let is_persistent = ul_sys::ulSessionIsPersistent(raw);

        Self {
            internal: raw,
            need_to_destroy: false,

            is_persistent,
            name,
            id,
            disk_path,
        }
    }

    /// Returns the underlying [`ul_sys::ULSession`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULSession {
        self.internal
    }
}

impl Session {
    /// A unique numeric ID identifying this session.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// A unique name identifying this session.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The disk path of this session (only valid for persistent sessions).
    pub fn disk_path(&self) -> &str {
        &self.disk_path
    }

    /// Whether or not this session is written to disk.
    pub fn is_persistent(&self) -> bool {
        self.is_persistent
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                ul_sys::ulDestroySession(self.internal);
            }
        }
    }
}

/// The `Renderer` manages all [`View`]s  and coordinates painting,
/// network requests, and event dispatch
///
/// You don't have to create this instance directly if you use the AppCore API.
/// The [`App`](crate::app::App) struct will automatically create a `Renderer`
/// and perform all rendering within its run loop.
pub struct Renderer {
    internal: ul_sys::ULRenderer,

    need_to_destroy: bool,

    default_session: Session,
}

impl Renderer {
    /// Internal helper to get a reference to the underlying Renderer.
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
    /// Unlike [`App::new`](crate::app::App::new), this does not use any native windows for drawing and allows you to manage
    /// your own runloop and painting. This method is recommended for those wishing to integrate the
    /// library into a game.
    ///
    /// This instance manages the lifetime of all [`View`]s and coordinates all painting, rendering,
    /// network requests, and event dispatch.
    ///
    /// You should only call this once per process lifetime.
    ///
    /// You shoud set up your platform handlers (eg,
    /// [`platform::set_gpu_driver`](crate::platform::set_gpu_driver),
    /// [`platform::set_logger`](crate::platform::set_logger),
    /// [`platform::enable_default_logger`](crate::platform::enable_default_logger),
    /// [`platform::enable_platform_file_system`](crate::platform::enable_platform_file_system),
    /// etc.) before calling this.
    ///
    /// You will also need to define a font loader before calling this --
    /// currently the only way to do this is
    /// [`platform::enable_platform_fontloader`](crate::platform::enable_platform_fontloader).
    ///
    /// You should not call this if you are using [`App::new`](crate::app::App::new),
    /// it creates its own renderer and provides default implementations for
    /// various platform handlers automatically.
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
    /// Update timers and dispatch internal callbacks. You should call this often
    /// from your main application loop.
    pub fn update(&self) {
        unsafe { ul_sys::ulUpdate(self.internal) };
    }

    /// Render all active views to their respective render-targets/surfaces.
    ///
    /// You should call this once per frame (usually in synchrony with the
    /// monitor's refresh rate).
    ///
    /// [`View`]s are only repainted if they actually need painting.
    /// (See [`View::needs_paint`](crate::view::View::needs_paint))
    pub fn render(&self) {
        unsafe { ul_sys::ulRender(self.internal) };
    }

    /// Attempt to release as much memory as possible.
    /// Don't call this from any callbacks or driver code.
    pub fn purge_memory(&self) {
        unsafe { ul_sys::ulPurgeMemory(self.internal) };
    }

    /// Print detailed memory usage statistics to the log.
    /// (See [`platform::set_logger`](crate::platform::set_logger) or
    /// [`platform::enable_default_logger`](crate::platform::enable_default_logger))
    pub fn log_memory_usage(&self) {
        unsafe { ul_sys::ulLogMemoryUsage(self.internal) };
    }

    /// Create a Session to store local data in (such as cookies, local storage,
    /// application cache, indexed db, etc).
    ///
    /// A default, persistent Session is already created for you. You only need to call this
    /// if you want to create private, in-memory session or use a separate session for each
    /// [`View`].
    ///
    /// # Arguments
    /// * `is_persistent` - Whether or not to store the session on disk.
    /// Persistent sessions will be written to the path set in
    /// [`ConfigBuilder::cache_path`](crate::config::ConfigBuilder::cache_path).
    /// * `name` -  A unique name for this session, this will be used to
    /// generate a unique disk path for persistent sessions.
    pub fn create_session(&self, is_persistent: bool, name: &str) -> Session {
        unsafe { Session::create(self.internal, is_persistent, name) }
    }

    /// Get the default Session. This session is persistent (backed to disk) and has the name
    /// "default".
    pub fn default_session(&self) -> &Session {
        &self.default_session
    }

    /// Create a new View.
    ///
    /// # Arguments
    /// * `width` - The initial width, in pixels.
    /// * `height` - The initial height, in pixels.
    /// * `config` - The configuration for the view.
    /// * `session` - The session to store local data in. Passing [`None`] will
    /// use the default session.
    pub fn create_view(
        &self,
        width: u32,
        height: u32,
        view_config: &ViewConfig,
        session: Option<&Session>,
    ) -> View {
        unsafe { View::create(self.internal, width, height, view_config, session) }
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
