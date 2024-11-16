//! The `Renderer` manages all [`View`]s  and coordinates painting,
//! network requests, and event dispatch
//!
//! [`Renderer`] should be used when you want to access GPU internals and textures
//! to integrate with your own rendering pipeline.
//!
//! Before creating a renderer [`Renderer::create`] you must supply a custom
//! [`GpuDriver`](crate::gpu_driver::GpuDriver) in
//! [`platform::set_gpu_driver`](crate::platform::set_gpu_driver).
use std::{ffi::CString, sync::Arc};

use crate::{
    config::Config,
    error::CreationError,
    event::{GamepadAxisEvent, GamepadButtonEvent, GamepadEvent},
    string::UlString,
    view::{View, ViewConfig},
    Library,
};

/// A Session stores local data such as cookies, local storage, and application
/// cache for one or more [`View`]s.
/// (See [`Renderer::create_session`](crate::renderer::Renderer::create_session))
pub struct Session {
    lib: Arc<Library>,
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
        lib: Arc<Library>,
        renderer: ul_sys::ULRenderer,
        is_persistent: bool,
        name: &str,
    ) -> Result<Self, CreationError> {
        let ul_string_name = UlString::from_str(lib.clone(), name)?;
        let internal =
            lib.ultralight()
                .ulCreateSession(renderer, is_persistent, ul_string_name.to_ul());

        if internal.is_null() {
            return Err(CreationError::NullReference);
        }

        let id = lib.ultralight().ulSessionGetId(internal);
        let disk_path =
            UlString::copy_raw_to_string(&lib, lib.ultralight().ulSessionGetDiskPath(internal))?;

        Ok(Self {
            lib,
            internal,
            need_to_destroy: true,

            is_persistent,
            name: name.to_string(),
            id,
            disk_path,
        })
    }

    /// Helper internal function to allow getting a reference to a managed
    /// session.
    pub(crate) unsafe fn from_raw(
        lib: Arc<Library>,
        raw: ul_sys::ULSession,
    ) -> Result<Self, CreationError> {
        if raw.is_null() {
            return Err(CreationError::NullReference);
        }

        let id = lib.ultralight().ulSessionGetId(raw);
        let disk_path =
            UlString::copy_raw_to_string(&lib, lib.ultralight().ulSessionGetDiskPath(raw))?;
        let name = UlString::copy_raw_to_string(&lib, lib.ultralight().ulSessionGetName(raw))?;
        let is_persistent = lib.ultralight().ulSessionIsPersistent(raw);

        Ok(Self {
            lib,
            internal: raw,
            need_to_destroy: false,

            is_persistent,
            name,
            id,
            disk_path,
        })
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
                self.lib.ultralight().ulDestroySession(self.internal);
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
    lib: Arc<Library>,
    internal: ul_sys::ULRenderer,

    need_to_destroy: bool,
    default_session: Session,
}

impl Renderer {
    /// Internal helper to get a reference to the underlying Renderer.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw(
        lib: Arc<Library>,
        raw: ul_sys::ULRenderer,
    ) -> Result<Self, CreationError> {
        let raw_default_session = lib.ultralight().ulDefaultSession(raw);
        if raw_default_session.is_null() {
            return Err(CreationError::NullReference);
        }
        let default_session = Session::from_raw(lib.clone(), raw_default_session)?;

        Ok(Self {
            lib,
            internal: raw,
            need_to_destroy: false,
            default_session,
        })
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
    /// You must set up your platform handlers (eg,
    /// [`platform::set_gpu_driver`](crate::platform::set_gpu_driver),
    /// [`platform::set_logger`](crate::platform::set_logger),
    /// [`platform::enable_default_logger`](crate::platform::enable_default_logger),
    /// [`platform::enable_platform_filesystem`](crate::platform::enable_platform_filesystem),
    /// etc.) before calling this.
    ///
    /// You will also need to define a font loader before calling this --
    /// currently the only way to do this is
    /// [`platform::enable_platform_fontloader`](crate::platform::enable_platform_fontloader).
    ///
    /// You should not call this if you are using [`App::new`](crate::app::App::new),
    /// it creates its own renderer and provides default implementations for
    /// various platform handlers automatically.
    pub fn create(config: Config) -> Result<Self, CreationError> {
        let lib = config.lib();
        let internal = unsafe { lib.ultralight().ulCreateRenderer(config.to_ul()) };
        if internal.is_null() {
            return Err(CreationError::NullReference);
        }
        let default_session =
            unsafe { Session::from_raw(lib.clone(), lib.ultralight().ulDefaultSession(internal)) }?;

        Ok(Self {
            lib: lib.clone(),
            internal,
            need_to_destroy: true,
            default_session,
        })
    }
}

impl Renderer {
    /// Update timers and dispatch internal callbacks. You should call this often
    /// from your main application loop.
    pub fn update(&self) {
        unsafe { self.lib.ultralight().ulUpdate(self.internal) };
    }

    /// Render all active views to their respective render-targets/surfaces.
    ///
    /// You should call this once per frame (usually in synchrony with the
    /// monitor's refresh rate).
    ///
    /// [`View`]s are only repainted if they actually need painting.
    /// (See [`View::needs_paint`](crate::view::View::needs_paint))
    pub fn render(&self) {
        unsafe { self.lib.ultralight().ulRender(self.internal) };
    }

    /// Attempt to release as much memory as possible.
    /// Don't call this from any callbacks or driver code.
    pub fn purge_memory(&self) {
        unsafe { self.lib.ultralight().ulPurgeMemory(self.internal) };
    }

    /// Print detailed memory usage statistics to the log.
    /// (See [`platform::set_logger`](crate::platform::set_logger) or
    /// [`platform::enable_default_logger`](crate::platform::enable_default_logger))
    pub fn log_memory_usage(&self) {
        unsafe { self.lib.ultralight().ulLogMemoryUsage(self.internal) };
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
    ///   Persistent sessions will be written to the path set in
    ///   [`ConfigBuilder::cache_path`](crate::config::ConfigBuilder::cache_path).
    /// * `name` -  A unique name for this session, this will be used to
    ///   generate a unique disk path for persistent sessions.
    pub fn create_session(
        &self,
        is_persistent: bool,
        name: &str,
    ) -> Result<Session, CreationError> {
        unsafe { Session::create(self.lib.clone(), self.internal, is_persistent, name) }
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
    ///   use the default session.
    pub fn create_view(
        &self,
        width: u32,
        height: u32,
        view_config: &ViewConfig,
        session: Option<&Session>,
    ) -> Option<View> {
        unsafe { View::create(self.internal, width, height, view_config, session) }
    }

    /// Start the remote inspector server.
    ///
    /// While the remote inspector is active, Views that are loaded into this renderer
    /// will be able to be remotely inspected from another Ultralight instance either locally
    /// (another app on same machine) or remotely (over the network) by navigating a View to:
    /// ```txt
    ///  inspector://<address>:<port>
    /// ```
    ///
    /// Returns `true` if the server was started successfully, `false` otherwise.
    pub fn start_remote_inspector_server(
        &self,
        address: &str,
        port: u16,
    ) -> Result<bool, CreationError> {
        unsafe {
            let c_str = CString::new(address)?;
            Ok(self.lib.ultralight().ulStartRemoteInspectorServer(
                self.internal,
                c_str.as_ptr(),
                port,
            ))
        }
    }

    /// Notify the renderer that a display has refreshed (you should call this after vsync).
    ///
    /// This updates animations, smooth scroll, and `window.requestAnimationFrame()` for all Views
    /// matching the display id.
    pub fn refresh_display(&self, display_id: u32) {
        unsafe {
            self.lib
                .ultralight()
                .ulRefreshDisplay(self.internal, display_id)
        }
    }

    /// Describe the details of a gamepad, to be used with FireGamepadEvent and related
    /// events below. This can be called multiple times with the same index if the details change.
    ///
    /// # Arguments
    /// * `index` - The unique index (or "connection slot") of the gamepad. For example,
    ///   controller #1 would be "1", controller #2 would be "2" and so on.
    /// * `id` - A string ID representing the device, this will be made available
    ///   in JavaScript as gamepad.id
    /// * `axis_count` - The number of axes on the device.
    /// * `button_count` - The number of buttons on the device
    pub fn set_gamepad_details(
        &self,
        index: u32,
        id: &str,
        axis_count: u32,
        button_count: u32,
    ) -> Result<(), CreationError> {
        unsafe {
            let ul_string_id = UlString::from_str(self.lib.clone(), id)?;

            self.lib.ultralight().ulSetGamepadDetails(
                self.internal,
                index,
                ul_string_id.to_ul(),
                axis_count,
                button_count,
            );
        }
        Ok(())
    }

    /// Fire a gamepad event (connection / disconnection).
    ///
    /// Note:  The gamepad should first be described via [`set_gamepad_details`][Self::set_gamepad_details] before calling this
    ///        function.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/Gamepad>
    pub fn fire_gamepad_event(&self, event: GamepadEvent) -> Result<(), CreationError> {
        unsafe {
            self.lib
                .ultralight()
                .ulFireGamepadEvent(self.internal, event.to_ul())
        };
        Ok(())
    }

    /// Fire a gamepad axis event (to be called when an axis value is changed).
    ///
    /// Note:  The gamepad should be connected via a call to [`fire_gamepad_event`][Self::fire_gamepad_event] before calling this function.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/Gamepad/axes>
    pub fn fire_gamepad_axis_event(&self, event: GamepadAxisEvent) -> Result<(), CreationError> {
        unsafe {
            self.lib
                .ultralight()
                .ulFireGamepadAxisEvent(self.internal, event.to_ul())
        };
        Ok(())
    }

    /// Fire a gamepad button event (to be called when a button value is changed).
    ///
    /// Note:  The gamepad should be connected via a call to [`fire_gamepad_event`][Self::fire_gamepad_event] before calling this function.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/Gamepad/axes>
    pub fn fire_gamepad_button_event(
        &self,
        event: GamepadButtonEvent,
    ) -> Result<(), CreationError> {
        unsafe {
            self.lib
                .ultralight()
                .ulFireGamepadButtonEvent(self.internal, event.to_ul())
        };
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                self.lib.ultralight().ulDestroyRenderer(self.internal);
            }
        }
    }
}
