//! Events that can be fired in [`View`](crate::view::View)s.

use crate::{error::CreationError, key_code::VirtualKeyCode, string::UlString};

#[derive(Clone, Copy)]
/// The type of the [`KeyEvent`].
pub enum KeyEventType {
    /// Raw Key-Down type. Use this when a physical key is pressed.
    ///
    /// You should use `RawKeyDown` for physical key presses since it allows the renderer to
    /// handle accelerator command translation.
    RawKeyDown = ul_sys::ULKeyEventType_kKeyEventType_RawKeyDown as isize,

    /// Key-Down event type. (Does not trigger accelerator commands in WebCore)
    /// (eg, Ctrl+C for copy is an accelerator command).
    ///
    /// You should probably use `RawKeyDown` instead when a physical key is pressed.
    /// This type is only here for historic compatibility with WebCore's key event types.
    KeyDown = ul_sys::ULKeyEventType_kKeyEventType_KeyDown as isize,

    /// Key-Up event type. Use this when a physical key is released.
    KeyUp = ul_sys::ULKeyEventType_kKeyEventType_KeyUp as isize,

    /// Character input event type. Use this when the OS generates text from a physical key being
    /// pressed (for example, this maps to `WM_CHAR` on Windows).
    Char = ul_sys::ULKeyEventType_kKeyEventType_Char as isize,
}

/// Modifiers that can be pressed with a key.
pub struct KeyEventModifiers {
    /// Whether or not an ALT key is down
    pub alt: bool,
    /// Whether or not a Control key is down
    pub ctrl: bool,
    /// Whether or not a meta key (Command-key on Mac, Windows-key on Win) is down
    pub meta: bool,
    /// Whether or not a Shift key is down
    pub shift: bool,
}

impl KeyEventModifiers {
    fn to_u32(&self) -> u32 {
        let mut n = 0;

        if self.alt {
            n |= 1 << 0;
        }
        if self.ctrl {
            n |= 1 << 1;
        }
        if self.meta {
            n |= 1 << 2;
        }
        if self.shift {
            n |= 1 << 3;
        }

        n
    }
}

/// Wrapper around all arguments needed to create a [`KeyEvent`].
pub struct KeyEventCreationInfo<'a, 'b> {
    /// The type of the event.
    pub ty: KeyEventType,

    /// The modifiers that were pressed with the key.
    pub modifiers: KeyEventModifiers,

    /// The virtual key-code associated with this keyboard event.
    /// This is either directly from the event (ie, WPARAM on Windows) or via a
    /// mapping function.
    pub virtual_key_code: VirtualKeyCode,

    /// The actual key-code generated by the platform.
    /// The DOM spec primarily uses Windows-equivalent codes
    /// (hence `virtual_key_code` above) but it helps to also specify the
    /// platform-specific key-code as well.
    pub native_key_code: i32,

    /// The actual text generated by this keyboard event.
    /// This is usually only a single character.
    pub text: &'a str,

    /// The text generated by this keyboard event before
    /// all modifiers except shift are applied. This is used internally for
    /// working out shortcut keys. This is usually only a single character.
    pub unmodified_text: &'b str,
    /// Whether or not this is a keypad event.
    pub is_keypad: bool,
    /// Whether or not this was generated as the result
    /// of an auto-repeat (eg, holding down a key)
    pub is_auto_repeat: bool,
    /// Whether or not the pressed key is a "system key".
    /// This is a Windows-only concept and should be "false" for all
    /// non-Windows platforms. For more information, see the following link:
    ///   <http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx>
    pub is_system_key: bool,
}

/// A generic keyboard event, that can be used to fire a key event in a
/// `view` by [`View::fire_key_event`](crate::view::View::fire_key_event).
pub struct KeyEvent {
    internal: ul_sys::ULKeyEvent,
}

impl KeyEvent {
    /// Create a new `KeyEvent`.
    pub fn new(creation_info: KeyEventCreationInfo) -> Result<KeyEvent, CreationError> {
        let ul_string_text = unsafe { UlString::from_str(creation_info.text) }?;
        let ul_string_unmodified_text =
            unsafe { UlString::from_str(creation_info.unmodified_text) }?;

        let internal = unsafe {
            ul_sys::ulCreateKeyEvent(
                creation_info.ty as u32,
                creation_info.modifiers.to_u32(),
                creation_info.virtual_key_code.into(),
                creation_info.native_key_code,
                ul_string_text.to_ul(),
                ul_string_unmodified_text.to_ul(),
                creation_info.is_keypad,
                creation_info.is_auto_repeat,
                creation_info.is_system_key,
            )
        };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULKeyEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULKeyEvent {
        self.internal
    }
}

impl Drop for KeyEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyKeyEvent(self.internal);
        }
    }
}

#[derive(Clone, Copy)]
/// The type of the [`MouseEvent`].
pub enum MouseEventType {
    /// Mouse moved event type
    MouseMoved = ul_sys::ULMouseEventType_kMouseEventType_MouseMoved as isize,
    /// Mouse button pressed event type
    MouseDown = ul_sys::ULMouseEventType_kMouseEventType_MouseDown as isize,
    /// Mouse button released event type
    MouseUp = ul_sys::ULMouseEventType_kMouseEventType_MouseUp as isize,
}

#[derive(Clone, Copy)]
/// The type of button that was pressed or released.
pub enum MouseButton {
    None = ul_sys::ULMouseButton_kMouseButton_None as isize,
    Left = ul_sys::ULMouseButton_kMouseButton_Left as isize,
    Middle = ul_sys::ULMouseButton_kMouseButton_Middle as isize,
    Right = ul_sys::ULMouseButton_kMouseButton_Right as isize,
}

/// A generic mouse event, that can be used to fire a key event in a
/// `view` by [`View::fire_mouse_event`](crate::view::View::fire_mouse_event).
pub struct MouseEvent {
    internal: ul_sys::ULMouseEvent,
}

impl MouseEvent {
    /// Create a new `MouseEvent`.
    ///
    /// # Arguments
    /// * `ty` - The type of the event.
    /// * `x` - The x-position of the mouse. relative to the view.
    /// * `y` - The y-position of the mouse. relative to the view.
    /// * `button` - The button that was pressed or released if any.
    pub fn new(
        ty: MouseEventType,
        x: i32,
        y: i32,
        button: MouseButton,
    ) -> Result<MouseEvent, CreationError> {
        let internal = unsafe { ul_sys::ulCreateMouseEvent(ty as u32, x, y, button as u32) };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULMouseEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULMouseEvent {
        self.internal
    }
}

impl Drop for MouseEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyMouseEvent(self.internal);
        }
    }
}

#[derive(Clone, Copy)]
/// The type of the [`ScrollEvent`].
pub enum ScrollEventType {
    /// The delta value is interpreted as number of pixels
    ScrollByPixel = ul_sys::ULScrollEventType_kScrollEventType_ScrollByPixel as isize,
    /// The delta value is interpreted as number of pages
    ScrollByPage = ul_sys::ULScrollEventType_kScrollEventType_ScrollByPage as isize,
}

/// A generic scroll event, that can be used to fire a key event in a
/// `view` by [`View::fire_scroll_event`](crate::view::View::fire_scroll_event).
pub struct ScrollEvent {
    internal: ul_sys::ULScrollEvent,
}

impl ScrollEvent {
    /// Create a new `ScrollEvent`.
    ///
    /// # Arguments
    /// * `ty` - The type of the event.
    /// * `delta_x` - The horizontal scroll amount.
    /// * `delta_y` - The vertical scroll amount.
    pub fn new(
        ty: ScrollEventType,
        delta_x: i32,
        delta_y: i32,
    ) -> Result<ScrollEvent, CreationError> {
        let internal = unsafe { ul_sys::ulCreateScrollEvent(ty as u32, delta_x, delta_y) };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULScrollEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULScrollEvent {
        self.internal
    }
}

impl Drop for ScrollEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyScrollEvent(self.internal);
        }
    }
}

#[derive(Clone, Copy)]
/// The type of the [`GamepadEvent`].
pub enum GamepadEventType {
    /// This event type should be fired when a gamepad is connected
    ///
    /// Note: You will need to previously declare the gamepad, its index, and details about
    ///  its axis and button layout via [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details]
    ///  prior to calling [`Renderer::fire_gamepad_event`][crate::renderer::Renderer::fire_gamepad_event].
    Connected = ul_sys::ULGamepadEventType_kGamepadEventType_Connected as isize,
    /// This event type should be fired when a gamepad is disconnected.
    Disconnected = ul_sys::ULGamepadEventType_kGamepadEventType_Disconnected as isize,
}

/// Event representing a change in gamepad connection state
///
/// See [`Renderer::fire_gamepad_event`][crate::renderer::Renderer::fire_gamepad_event].
pub struct GamepadEvent {
    internal: ul_sys::ULGamepadEvent,
}

impl GamepadEvent {
    /// Create a new `GamepadEvent`.
    ///
    /// # Arguments
    /// * `index` - The index of the gamepad, this should match the value previously set in
    ///   [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details].
    /// * `ty` - The type of this GamepadEvent.
    pub fn new(index: u32, ty: GamepadEventType) -> Result<GamepadEvent, CreationError> {
        let internal = unsafe { ul_sys::ulCreateGamepadEvent(index, ty as u32) };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULGamepadEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULGamepadEvent {
        self.internal
    }
}

impl Drop for GamepadEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyGamepadEvent(self.internal);
        }
    }
}

/// Event representing a change in gamepad axis state (eg, pressing a stick in a certain direction).
///
/// See [`Renderer::fire_gamepad_axis_event`][crate::renderer::Renderer::fire_gamepad_axis_event].
pub struct GamepadAxisEvent {
    internal: ul_sys::ULGamepadAxisEvent,
}

impl GamepadAxisEvent {
    /// Create a new `GamepadAxisEvent`.
    ///
    /// # Arguments
    /// * `index` - The index of the gamepad, this should match the value previously set in
    ///   [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details].
    /// * `axis_index` - The index of the axis whose value has changed.
    ///   This value should be in the range previously set in [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details].
    /// * `value` - The new value of the axis. This value should be normalized to the range [-1.0, 1.0].
    pub fn new(index: u32, axis_index: u32, value: f64) -> Result<GamepadAxisEvent, CreationError> {
        let internal = unsafe { ul_sys::ulCreateGamepadAxisEvent(index, axis_index, value) };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULGamepadAxisEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULGamepadAxisEvent {
        self.internal
    }
}

impl Drop for GamepadAxisEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyGamepadAxisEvent(self.internal);
        }
    }
}

/// Event representing a change in gamepad button state (eg, pressing a button on a gamepad).
///
/// See [`Renderer::fire_gamepad_button_event`][crate::renderer::Renderer::fire_gamepad_button_event].
pub struct GamepadButtonEvent {
    internal: ul_sys::ULGamepadButtonEvent,
}

impl GamepadButtonEvent {
    /// Create a new `GamepadButtonEvent`.
    ///
    /// # Arguments
    /// * `index` - The index of the gamepad, this should match the value previously set in
    ///   [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details].
    /// * `button_index` - The index of the button whose value has changed.
    ///   This value should be in the range previously set in [`Renderer::set_gamepad_details`][crate::renderer::Renderer::set_gamepad_details].
    /// * `value` - The new value of the axis. This value should be normalized to the range [-1.0, 1.0].
    ///   with any value greater than 0.0 to be considered "pressed".
    pub fn new(
        index: u32,
        button_index: u32,
        value: f64,
    ) -> Result<GamepadButtonEvent, CreationError> {
        let internal = unsafe { ul_sys::ulCreateGamepadButtonEvent(index, button_index, value) };

        if internal.is_null() {
            Err(CreationError::NullReference)
        } else {
            Ok(Self { internal })
        }
    }

    /// Returns the underlying [`ul_sys::ULGamepadButtonEvent`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULGamepadButtonEvent {
        self.internal
    }
}

impl Drop for GamepadButtonEvent {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyGamepadButtonEvent(self.internal);
        }
    }
}
