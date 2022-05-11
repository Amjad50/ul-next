use std::slice;

/// A rust wrapper around [`ul_sys::ULString`], which is used in ultralight
/// functions.
pub(crate) struct UlString {
    internal: ul_sys::ULString,
}

impl UlString {
    /// Creates a new `UlString` from a `&str`.
    pub(crate) unsafe fn from_str(s: &str) -> Self {
        let internal =
            ul_sys::ulCreateStringUTF8(s.as_bytes().as_ptr() as *const i8, s.len() as u64);
        Self { internal }
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
    pub(crate) unsafe fn copy_raw_to_string(raw: ul_sys::ULString) -> String {
        let utf8_data = slice::from_raw_parts(
            ul_sys::ulStringGetData(raw),
            ul_sys::ulStringGetLength(raw) as usize,
        )
        .iter()
        .map(|c| *c as u8)
        .collect();

        // TODO: handle error
        String::from_utf8(utf8_data).unwrap()
    }
}

impl Drop for UlString {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyString(self.internal);
        }
    }
}
