use core::fmt;
use std::sync::Arc;

use crate::Library;

use super::{JSObject, JSString, JSValue};

/// JavaScript execution context.
///
/// This struct represents a JavaScript execution context. It is the top-level
/// object for evaluating JavaScript code.
///
/// Can be obtained initially from [`View::lock_js_context`](crate::view::View::lock_js_context).
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

    /// Create a new JavaScript execution context.
    pub fn new(lib: Arc<Library>) -> Self {
        let ctx = unsafe { lib.ultralight().JSGlobalContextCreate(std::ptr::null_mut()) };

        Self { internal: ctx, lib }
    }

    /// Get the global object for this context.
    pub fn global_object(&self) -> JSObject {
        JSObject::copy_from_raw(self, unsafe {
            self.lib
                .ultralight()
                .JSContextGetGlobalObject(self.internal)
        })
    }

    /// Get the global context for this context.
    pub fn global_context(&self) -> JSContext {
        JSContext::copy_from_raw(self.lib.clone(), unsafe {
            self.lib
                .ultralight()
                .JSContextGetGlobalContext(self.internal)
        })
    }

    /// Get the name of this context.
    ///
    /// A JSGlobalContext's name is exposed when inspecting the context to
    /// make it easier to identify the context you would like to inspect.
    ///
    /// The context may not have a name, in which case this method will return [`None`].
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

    /// Gets whether the context is inspectable in Web Inspector.
    pub fn is_inspectable(&self) -> bool {
        unsafe {
            self.lib
                .ultralight()
                .JSGlobalContextIsInspectable(self.internal as _)
        }
    }

    /// Evaluate a JavaScript script in this context.
    ///
    /// If an exception is thrown during evaluation, it will be returned as an
    /// [`Err`] value, otherwise the result of the script evaluation will be
    /// returned as an [`Ok`] value.
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
            Err(JSValue::copy_from_raw(self, exception))
        } else if ret.is_null() {
            Err(JSValue::new_string(self, "Failed to evaluate script"))
        } else {
            Ok(JSValue::copy_from_raw(self, ret))
        }
    }

    /// Checks for syntax errors in a string of JavaScript.
    ///
    /// `true` if the script is syntactically correct, otherwise `false`.
    pub fn check_script_syntax(&self, script: &str) -> Result<bool, JSValue> {
        let script = JSString::new(self.lib.clone(), script);
        let mut exception = std::ptr::null();
        let ret = unsafe {
            self.lib.ultralight().JSCheckScriptSyntax(
                self.internal,
                script.internal,
                std::ptr::null_mut(),
                0,
                &mut exception,
            )
        };

        if !exception.is_null() {
            return Err(JSValue::copy_from_raw(self, exception));
        }

        Ok(ret)
    }

    /// Performs a JavaScript garbage collection.
    ///
    /// JavaScript values that are on the machine stack, in a register,
    /// protected by `JSValueProtect`, set as the global object of an execution context,
    /// or reachable from any such value will not be collected.
    ///
    /// During JavaScript execution, you are not required to call this function; the
    /// JavaScript engine will garbage collect as needed. JavaScript values created
    /// within a context group are automatically destroyed when the last reference
    /// to the context group is released.
    pub fn garbage_collect(&self) {
        unsafe {
            self.lib.ultralight().JSGarbageCollect(self.internal);
        }
    }
}

impl fmt::Debug for JSContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSContext")
            .field("name", &self.name())
            .finish()
    }
}

impl Clone for JSContext {
    fn clone(&self) -> Self {
        Self::copy_from_raw(self.lib.clone(), self.internal)
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
