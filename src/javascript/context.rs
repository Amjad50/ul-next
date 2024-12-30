use std::sync::Arc;

use crate::Library;

use super::{JSObject, JSString, JSValue};

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

    pub fn global_object(&self) -> JSObject {
        JSObject::copy_from_raw(self, unsafe {
            self.lib
                .ultralight()
                .JSContextGetGlobalObject(self.internal)
        })
    }

    pub fn global_context(&self) -> JSContext {
        JSContext::copy_from_raw(self.lib.clone(), unsafe {
            self.lib
                .ultralight()
                .JSContextGetGlobalContext(self.internal)
        })
    }

    pub fn name(&self) -> Option<JSString> {
        let name = unsafe {
            self.lib
                .ultralight()
                .JSGlobalContextCopyName(self.internal as _)
        };
        if name.is_null() {
            return None;
        }
        Some(JSString::from_raw(self.lib.clone(), name))
    }

    pub fn is_inspectable(&self) -> bool {
        unsafe {
            self.lib
                .ultralight()
                .JSGlobalContextIsInspectable(self.internal as _)
        }
    }

    pub fn evaluate_script(&self, script: &str) -> Result<JSValue, JSValue> {
        let script = JSString::new(self.lib.clone(), script);
        let mut exception = std::ptr::null();
        let ret = unsafe {
            self.lib.ultralight().JSEvaluateScript(
                self.internal,
                script.internal,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception,
            )
        };

        if !exception.is_null() {
            return Err(JSValue::copy_from_raw(self, exception));
        } else if ret.is_null() {
            Err(JSValue::new_string(self, "Failed to evaluate script"))
        } else {
            Ok(JSValue::copy_from_raw(self, ret))
        }
    }

    pub fn garbage_collect(&self) {
        unsafe {
            self.lib.ultralight().JSGarbageCollect(self.internal);
        }
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
