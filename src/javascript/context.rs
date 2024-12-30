use std::sync::Arc;

use crate::Library;

pub struct JSContext {
    pub(crate) internal: ul_sys::JSContextRef,
    pub(crate) lib: Arc<Library>,
}

impl JSContext {
    pub(crate) fn copy_from_raw(lib: Arc<Library>, ctx: ul_sys::JSContextRef) -> Self {
        assert!(!ctx.is_null());

        let ctx = unsafe { lib.ultralight().JSGlobalContextRetain(ctx as _) };

        Self { internal: ctx, lib }
    }

    pub fn new(lib: Arc<Library>) -> Self {
        let ctx = unsafe { lib.ultralight().JSGlobalContextCreate(std::ptr::null_mut()) };

        Self { internal: ctx, lib }
    }
}

impl Drop for JSContext {
    fn drop(&mut self) {
        unsafe {
            self.lib
                .ultralight()
                .JSGlobalContextRelease(self.internal as _);
        }
    }
}
