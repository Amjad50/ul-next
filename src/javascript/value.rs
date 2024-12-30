use core::fmt;

use super::{JSContext, JSString};

#[repr(u32)]
pub enum JSType {
    Undefined = ul_sys::JSType_kJSTypeUndefined,
    Null = ul_sys::JSType_kJSTypeNull,
    Boolean = ul_sys::JSType_kJSTypeBoolean,
    Number = ul_sys::JSType_kJSTypeNumber,
    String = ul_sys::JSType_kJSTypeString,
    Object = ul_sys::JSType_kJSTypeObject,
    Symbol = ul_sys::JSType_kJSTypeSymbol,
}

pub struct JSValue<'a> {
    pub(crate) internal: ul_sys::JSValueRef,
    pub(crate) ctx: &'a JSContext,
}

impl<'a> JSValue<'a> {
    pub(crate) fn from_raw(ctx: &'a JSContext, value: ul_sys::JSValueRef) -> Self {
        assert!(!value.is_null());

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn copy_from_raw(ctx: &'a JSContext, value: ul_sys::JSValueRef) -> Self {
        assert!(!value.is_null());

        unsafe {
            ctx.lib.ultralight().JSValueProtect(ctx.internal, value);
        }

        Self::from_raw(ctx, value)
    }

    pub(crate) fn into_raw(self) -> ul_sys::JSValueRef {
        // add protection so that the `Drop` impl doesn't free while we need it
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueProtect(self.ctx.internal, self.internal);
        }

        self.internal
    }
}

impl<'a> JSValue<'a> {
    pub fn new_undefined(ctx: &'a JSContext) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeUndefined(ctx.internal) };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_null(ctx: &'a JSContext) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeNull(ctx.internal) };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_boolean(ctx: &'a JSContext, value: bool) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeBoolean(ctx.internal, value) };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_number(ctx: &'a JSContext, value: f64) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeNumber(ctx.internal, value) };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_symbol(ctx: &'a JSContext, value: &str) -> Self {
        let value = JSString::new(ctx.lib.clone(), value);

        let value = unsafe {
            ctx.lib
                .ultralight()
                .JSValueMakeSymbol(ctx.internal, value.internal)
        };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_string(ctx: &'a JSContext, value: &str) -> Self {
        let value = JSString::new(ctx.lib.clone(), value);

        let value = unsafe {
            ctx.lib
                .ultralight()
                .JSValueMakeString(ctx.internal, value.internal)
        };

        Self {
            internal: value,
            ctx,
        }
    }

    pub fn new_from_json(ctx: &'a JSContext, value: &str) -> Option<Self> {
        let value = JSString::new(ctx.lib.clone(), value);

        let value = unsafe {
            ctx.lib
                .ultralight()
                .JSValueMakeFromJSONString(ctx.internal, value.internal)
        };

        if value.is_null() {
            None
        } else {
            Some(Self {
                internal: value,
                ctx,
            })
        }
    }
}

impl<'a> JSValue<'a> {
    pub fn get_type(&self) -> JSType {
        let ty = unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueGetType(self.ctx.internal, self.internal)
        };

        match ty {
            ul_sys::JSType_kJSTypeUndefined => JSType::Undefined,
            ul_sys::JSType_kJSTypeNull => JSType::Null,
            ul_sys::JSType_kJSTypeBoolean => JSType::Boolean,
            ul_sys::JSType_kJSTypeNumber => JSType::Number,
            ul_sys::JSType_kJSTypeString => JSType::String,
            ul_sys::JSType_kJSTypeObject => JSType::Object,
            ul_sys::JSType_kJSTypeSymbol => JSType::Symbol,
            _ => panic!("Unknown JSValue type: {}", ty),
        }
    }

    pub fn is_undefined(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsUndefined(self.ctx.internal, self.internal)
        }
    }

    pub fn is_null(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsNull(self.ctx.internal, self.internal)
        }
    }

    pub fn is_date(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsDate(self.ctx.internal, self.internal)
        }
    }

    pub fn is_array(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsArray(self.ctx.internal, self.internal)
        }
    }

    pub fn is_symbol(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsSymbol(self.ctx.internal, self.internal)
        }
    }

    pub fn is_object(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsObject(self.ctx.internal, self.internal)
        }
    }

    pub fn is_string(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsString(self.ctx.internal, self.internal)
        }
    }

    pub fn is_number(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsNumber(self.ctx.internal, self.internal)
        }
    }

    pub fn is_boolean(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsBoolean(self.ctx.internal, self.internal)
        }
    }
}

impl fmt::Debug for JSValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_undefined() {
            write!(f, "Value::undefined({:p})", self.internal)
        } else {
            write!(f, "JSValue({:p})", self.internal)
        }
    }
}

impl Drop for JSValue<'_> {
    fn drop(&mut self) {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueUnprotect(self.ctx.internal, self.internal);
        }
    }
}
