use crate::string::UlString;

#[derive(Clone, Copy)]
pub enum KeyEventType {
    /// Raw Key-Down type. Use this when a physical key is pressed.
    ///
    /// @NOTE: You should use RawKeyDown for physical key presses since it allows the renderer to
    ///        handle accelerator command translation.
    RawKeyDown = ul_sys::ULKeyEventType_kKeyEventType_RawKeyDown as isize,

    /// Key-Down event type. (Does not trigger accelerator commands in WebCore)
    ///
    /// @NOTE: You should probably use RawKeyDown instead when a physical key is pressed.
    ///        This type is only here for historic compatibility with WebCore's key event types.
    KeyDown = ul_sys::ULKeyEventType_kKeyEventType_KeyDown as isize,

    /// Key-Up event type. Use this when a physical key is released.
    KeyUp = ul_sys::ULKeyEventType_kKeyEventType_KeyUp as isize,

    /// Character input event type. Use this when the OS generates text from a physical key being
    /// pressed (for example, this maps to WM_CHAR on Windows).
    Char = ul_sys::ULKeyEventType_kKeyEventType_Char as isize,
}

pub struct KeyEventModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
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

pub struct KeyEvent {
    ty: KeyEventType,
    modifiers: KeyEventModifiers,
    virtual_key_code: i32,
    native_key_code: i32,
    text: String,
    unmodified_text: String,
    is_keypad: bool,
    is_auto_repeat: bool,
    is_system_key: bool,

    internal: ul_sys::ULKeyEvent,
}

impl KeyEvent {
    pub fn new(
        ty: KeyEventType,
        modifiers: KeyEventModifiers,
        virtual_key_code: i32,
        native_key_code: i32,
        text: &str,
        unmodified_text: &str,
        is_keypad: bool,
        is_auto_repeat: bool,
        is_system_key: bool,
    ) -> KeyEvent {
        let ul_string_text = unsafe { UlString::from_str(text) };
        let ul_string_unmodified_text = unsafe { UlString::from_str(unmodified_text) };

        let internal = unsafe {
            ul_sys::ulCreateKeyEvent(
                ty as u32,
                modifiers.to_u32(),
                virtual_key_code,
                native_key_code,
                ul_string_text.to_ul(),
                ul_string_unmodified_text.to_ul(),
                is_keypad,
                is_auto_repeat,
                is_system_key,
            )
        };

        Self {
            ty,
            modifiers,
            virtual_key_code,
            native_key_code,
            text: text.to_string(),
            unmodified_text: unmodified_text.to_string(),
            is_keypad,
            is_auto_repeat,
            is_system_key,
            internal,
        }
    }

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
pub enum MouseEventType {
    MouseMoved = ul_sys::ULMouseEventType_kMouseEventType_MouseMoved as isize,
    MouseDown = ul_sys::ULMouseEventType_kMouseEventType_MouseDown as isize,
    MouseUp = ul_sys::ULMouseEventType_kMouseEventType_MouseUp as isize,
}

#[derive(Clone, Copy)]
pub enum MouseButton {
    None = ul_sys::ULMouseButton_kMouseButton_None as isize,
    Left = ul_sys::ULMouseButton_kMouseButton_Left as isize,
    Middle = ul_sys::ULMouseButton_kMouseButton_Middle as isize,
    Right = ul_sys::ULMouseButton_kMouseButton_Right as isize,
}

pub struct MouseEvent {
    ty: MouseEventType,
    x: i32,
    y: i32,
    button: MouseButton,

    internal: ul_sys::ULMouseEvent,
}

impl MouseEvent {
    pub fn new(ty: MouseEventType, x: i32, y: i32, button: MouseButton) -> MouseEvent {
        let internal = unsafe { ul_sys::ulCreateMouseEvent(ty as u32, x, y, button as u32) };

        Self {
            ty,
            x,
            y,
            button,
            internal,
        }
    }

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
pub enum ScrollEventType {
    ScrollByPixel = ul_sys::ULScrollEventType_kScrollEventType_ScrollByPixel as isize,
    ScrollByPage = ul_sys::ULScrollEventType_kScrollEventType_ScrollByPage as isize,
}

pub struct ScrollEvent {
    ty: ScrollEventType,
    delta_x: i32,
    delta_y: i32,

    internal: ul_sys::ULScrollEvent,
}

impl ScrollEvent {
    pub fn new(ty: ScrollEventType, delta_x: i32, delta_y: i32) -> ScrollEvent {
        let internal = unsafe { ul_sys::ulCreateScrollEvent(ty as u32, delta_x, delta_y) };

        Self {
            ty,
            delta_x,
            delta_y,
            internal,
        }
    }

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
