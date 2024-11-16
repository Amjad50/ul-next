use std::{os::raw::c_char, slice, sync::Arc};

use crate::{error::CreationError, Library};

/// A rust wrapper around [`ul_sys::ULString`], which is used in ultralight
/// functions.
pub(crate) struct UlString {
    lib: Arc<Library>,
    internal: ul_sys::ULString,
}

impl UlString {
    /// Creates a new `UlString` from a `&str`.
    pub(crate) unsafe fn from_str(lib: Arc<Library>, s: &str) -> Result<Self, CreationError> {
        let internal = Self::from_str_unmanaged(&lib, s)?;
        Ok(Self { lib, internal })
    }

    /// Creates a new `UlString` from a `&str`. But will not destroy on drop.
    /// This will be sent to `Ultralight` library and it will handle its memory.
    pub(crate) unsafe fn from_str_unmanaged(
        lib: &Arc<Library>,
        s: &str,
    ) -> Result<ul_sys::ULString, CreationError> {
        let internal = lib
            .ultralight()
            .ulCreateStringUTF8(s.as_bytes().as_ptr() as *const c_char, s.len());
        if internal.is_null() {
            Err(CreationError::UlStringCreationError(s.to_string()))
        } else {
            Ok(internal)
        }
    }

    /// Returns the underlying [`ul_sys::ULString`] struct, to be used locally for
    /// calling the underlying C API.
    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULString {
        self.internal
    }

    /// create a rust String copy without destroying the original
    ///
    /// Some ultralight APIs owns the string, so we can't destroy it, its always
    /// safer to make our own copy.
    pub(crate) unsafe fn copy_raw_to_string(
        lib: &Arc<Library>,
        raw: ul_sys::ULString,
    ) -> Result<String, CreationError> {
        if raw.is_null() {
            return Err(CreationError::NullReference);
        }

        let raw_data = lib.ultralight().ulStringGetData(raw);
        if raw_data.is_null() {
            return Err(CreationError::NullReference);
        }
        let utf8_data = slice::from_raw_parts(raw_data, lib.ultralight().ulStringGetLength(raw))
            .iter()
            .map(|c| *c as u8)
            .collect();

        String::from_utf8(utf8_data).map_err(|e| e.into())
    }
}

impl Drop for UlString {
    fn drop(&mut self) {
        unsafe {
            self.lib.ultralight().ulDestroyString(self.internal);
        }
    }
}
