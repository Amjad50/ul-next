use std::slice;

pub(crate) struct UlString {
    internal: ul_sys::ULString,
}

impl UlString {
    pub(crate) unsafe fn from_raw(internal: ul_sys::ULString) -> Self {
        Self { internal }
    }

    pub(crate) unsafe fn from_str(s: &str) -> Self {
        let internal =
            ul_sys::ulCreateStringUTF8(s.as_bytes().as_ptr() as *const i8, s.len() as u64);
        Self { internal }
    }

    pub(crate) unsafe fn to_string(&self) -> String {
        Self::copy_raw_to_string(self.internal)
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULString {
        self.internal
    }

    // create a rust String copy without destroying the original
    pub(crate) unsafe fn copy_raw_to_string(raw: ul_sys::ULString) -> String {
        let utf16_data = slice::from_raw_parts(
            ul_sys::ulStringGetData(raw),
            ul_sys::ulStringGetLength(raw) as usize,
        );

        // TODO: handle error
        String::from_utf16(utf16_data).unwrap()
    }
}

impl Drop for UlString {
    fn drop(&mut self) {
        unsafe {
            ul_sys::ulDestroyString(self.internal);
        }
    }
}
