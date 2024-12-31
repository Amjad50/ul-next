use core::fmt;
use std::ops::Deref;

use super::{JSContext, JSObject, JSString, JSTypedArray, JSTypedArrayType};

/// A trait for converting a type into a [`JSValue`].
///
/// Types implementing this trait are [`JSValue`], we are wrapping them in other types
/// to provide specific functionality.
pub trait AsJSValue<'a>: Deref<Target = JSValue<'a>> + AsRef<JSValue<'a>> {
    fn into_value(self) -> JSValue<'a>;
    fn as_value(&self) -> &JSValue<'a>;
}

/// An enum identifying the type of a [`JSValue`].
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JSType {
    Undefined = ul_sys::JSType_kJSTypeUndefined,
    Null = ul_sys::JSType_kJSTypeNull,
    Boolean = ul_sys::JSType_kJSTypeBoolean,
    Number = ul_sys::JSType_kJSTypeNumber,
    String = ul_sys::JSType_kJSTypeString,
    Object = ul_sys::JSType_kJSTypeObject,
    Symbol = ul_sys::JSType_kJSTypeSymbol,
}

/// A JavaScript value.
///
/// This is the basic type for managing values in JavaScriptCore. It can represent
/// any JavaScript value, including `undefined`, `null`, booleans, numbers, strings,
/// symbols, objects, and arrays.
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

    pub(crate) fn copy_from_raw(ctx: &'a JSContext, value: ul_sys::JSValueRef) -> Self {
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
    /// Creates a Javascript `undefined` value.
    pub fn new_undefined(ctx: &'a JSContext) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeUndefined(ctx.internal) };

        Self {
            internal: value,
            ctx,
        }
    }

    /// Creates a Javascript `null` value.
    pub fn new_null(ctx: &'a JSContext) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeNull(ctx.internal) };

        Self {
            internal: value,
            ctx,
        }
    }

    /// Creates a Javascript boolean value.
    pub fn new_boolean(ctx: &'a JSContext, value: bool) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeBoolean(ctx.internal, value) };

        Self {
            internal: value,
            ctx,
        }
    }

    /// Creates a Javascript numeric value.
    pub fn new_number(ctx: &'a JSContext, value: f64) -> Self {
        let value = unsafe { ctx.lib.ultralight().JSValueMakeNumber(ctx.internal, value) };

        Self {
            internal: value,
            ctx,
        }
    }

    /// Creates a unique Javascript symbol object.
    ///
    /// `description` is a string that describes the symbol.
    pub fn new_symbol(ctx: &'a JSContext, description: &str) -> Self {
        let value = JSString::new(ctx.lib.clone(), description);

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

    /// Converts a Javascript string object to a Javascript value.
    pub fn from_jsstring(ctx: &'a JSContext, value: JSString) -> Self {
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

    /// Creates a Javascript string value from a Rust string.
    ///
    /// If you have already `JSString` object, use [`JSValue::from_jsstring`] instead.
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

    /// Creates a JavaScript value from a JSON formatted string.
    ///
    /// Returns [`None`] if the JSON string is invalid.
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

impl JSValue<'_> {
    /// Returns a JavaScript value's type.
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

    /// Returns `true` if the value is `undefined`.
    pub fn is_undefined(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsUndefined(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is `null`.
    pub fn is_null(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsNull(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a JavaScript date object.
    pub fn is_date(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsDate(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a JavaScript array.
    pub fn is_array(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsArray(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value's type is the symbol type
    pub fn is_symbol(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsSymbol(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is an object.
    pub fn is_object(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsObject(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a string.
    pub fn is_string(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsString(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a number.
    pub fn is_number(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsNumber(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a boolean.
    pub fn is_boolean(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueIsBoolean(self.ctx.internal, self.internal)
        }
    }

    /// Returns `true` if the value is a Typed Array.
    pub fn is_typed_array(&self) -> bool {
        // Note: we are creating the object `JSTypedArray` here to check if the value is a typed
        // array, i.e. it shouldn't be used to call any other `JSTypedArray` methods.
        let typed_array = JSTypedArray {
            value: Self {
                internal: self.internal,
                ctx: self.ctx,
            },
        };

        match typed_array.ty().ok() {
            None | Some(JSTypedArrayType::None) => false,
            Some(_) => true,
        }
    }
}

impl<'a> JSValue<'a> {
    /// Converts a JavaScript value to object.
    ///
    /// Returns an [`Err`] if an exception is thrown.
    pub fn as_object(&self) -> Result<JSObject<'a>, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSValueToObject(
                self.ctx.internal,
                self.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result.is_null() {
            Err(JSValue::new_string(
                self.ctx,
                "Failed to convert value to object",
            ))
        } else {
            Ok(JSObject {
                value: JSValue::from_raw(self.ctx, result),
            })
        }
    }

    /// Converts a JavaScript value to string.
    ///
    /// Returns an [`Err`] if an exception is thrown.
    pub fn as_string(&self) -> Result<JSString, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSValueToStringCopy(
                self.ctx.internal,
                self.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result.is_null() {
            Err(JSValue::new_string(
                self.ctx,
                "Failed to convert value to string",
            ))
        } else {
            Ok(JSString::copy_from_raw(self.ctx.lib.clone(), result))
        }
    }

    /// Converts a JavaScript value to number.
    ///
    /// Returns an [`Err`] if an exception is thrown.
    pub fn as_number(&self) -> Result<f64, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSValueToNumber(
                self.ctx.internal,
                self.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(result)
        }
    }

    /// Converts a JavaScript value to boolean.
    pub fn as_boolean(&self) -> bool {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueToBoolean(self.ctx.internal, self.internal)
        }
    }

    /// Converts a JavaScript value to a typed array.
    ///
    /// Returns an [`Err`] if the value is not a typed array, or if an exception is thrown.
    pub fn as_typed_array(&self) -> Result<JSTypedArray<'a>, JSValue<'a>> {
        if self.is_typed_array() {
            let object = self.as_object()?;

            Ok(JSTypedArray {
                value: object.value,
            })
        } else {
            Err(JSValue::new_string(self.ctx, "Value is not a typed array"))
        }
    }

    /// Converts a JavaScript value to JSON serialized representation of a JS value.
    ///
    /// Returns an [`Err`] if an exception is thrown.
    pub fn to_json_string(&self) -> Result<JSString, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSValueCreateJSONString(
                self.ctx.internal,
                self.internal,
                0,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result.is_null() {
            Err(JSValue::new_string(
                self.ctx,
                "Failed to convert value to JSON string",
            ))
        } else {
            Ok(JSString::from_raw(self.ctx.lib.clone(), result))
        }
    }
}

impl Clone for JSValue<'_> {
    fn clone(&self) -> Self {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSValueProtect(self.ctx.internal, self.internal)
        };

        Self {
            internal: self.internal,
            ctx: self.ctx,
        }
    }
}

impl fmt::Debug for JSValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSValue")
            .field("type", &self.get_type())
            .field("repr", &self.to_json_string())
            .finish()
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
