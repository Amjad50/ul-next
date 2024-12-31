use core::fmt;
use std::{ffi::c_char, sync::Arc};

use crate::Library;

/// A JavaScript string.
///
/// Its internally a UTF16 character buffer.
pub struct JSString {
    pub(crate) internal: ul_sys::JSStringRef,
    lib: Arc<Library>,
}

impl JSString {
    pub(crate) fn from_raw(lib: Arc<Library>, string: ul_sys::JSStringRef) -> Self {
        assert!(!string.is_null());

        Self {
            internal: string,
            lib,
        }
    }

    pub(crate) fn copy_from_raw(
        lib: Arc<Library>,
        string: *mut ul_sys::OpaqueJSString,
    ) -> JSString {
        assert!(!string.is_null());

        let string = unsafe { lib.ultralight().JSStringRetain(string) };

        Self {
            internal: string,
            lib,
        }
    }

    /// Creates a new JavaScript string from a Rust string.
    pub fn new(lib: Arc<Library>, string: &str) -> Self {
        let cstring = std::ffi::CString::new(string).unwrap();

        let string = unsafe {
            lib.ultralight()
                .JSStringCreateWithUTF8CString(cstring.as_ptr())
        };

        Self {
            internal: string,
            lib,
        }
    }

    /// Returns `true` if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of Unicode characters in the JavaScript string.
    pub fn len(&self) -> usize {
        unsafe { self.lib.ultralight().JSStringGetLength(self.internal) }
    }
}

impl From<&JSString> for String {
    fn from(string: &JSString) -> Self {
        let max_size = unsafe {
            string
                .lib
                .ultralight()
                .JSStringGetMaximumUTF8CStringSize(string.internal)
        };

        let mut buffer: Vec<u8> = Vec::with_capacity(max_size);

        unsafe {
            let actual_size = string.lib.ultralight().JSStringGetUTF8CString(
                string.internal,
                buffer.as_mut_ptr().cast::<c_char>(),
                max_size,
            );
            buffer.set_len(actual_size - 1);
        }

        String::from_utf8(buffer).unwrap()
    }
}

impl Clone for JSString {
    fn clone(&self) -> Self {
        let string = unsafe { self.lib.ultralight().JSStringRetain(self.internal) };

        Self {
            internal: string,
            lib: self.lib.clone(),
        }
    }
}

impl fmt::Debug for JSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        String::from(self).fmt(f)
    }
}

impl fmt::Display for JSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        String::from(self).fmt(f)
    }
}

impl Drop for JSString {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().JSStringRelease(self.internal);
        }
    }
}
