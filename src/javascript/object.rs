use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

use crate::Library;

use super::{JSContext, JSString, JSValue};

// TODO: major hack, not sure how to get access to the Library
//       from inside the trampoline
static LIBRARY: OnceLock<Arc<Library>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JSPropertyAttributes {
    pub read_only: bool,
    pub dont_enum: bool,
    pub dont_delete: bool,
}

impl JSPropertyAttributes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn dont_enum(mut self, dont_enum: bool) -> Self {
        self.dont_enum = dont_enum;
        self
    }

    pub fn dont_delete(mut self, dont_delete: bool) -> Self {
        self.dont_delete = dont_delete;
        self
    }

    pub(crate) fn to_raw(&self) -> u32 {
        let mut raw = 0;

        if self.read_only {
            raw |= ul_sys::kJSPropertyAttributeReadOnly;
        }

        if self.dont_enum {
            raw |= ul_sys::kJSPropertyAttributeDontEnum;
        }

        if self.dont_delete {
            raw |= ul_sys::kJSPropertyAttributeDontDelete;
        }

        raw
    }
}

pub struct JSObject<'a> {
    value: JSValue<'a>,
}

impl<'a> JSObject<'a> {
    pub(crate) fn copy_from_raw(ctx: &'a JSContext, obj: ul_sys::JSObjectRef) -> Self {
        assert!(!obj.is_null());

        // add one
        unsafe { ctx.lib.ultralight().JSValueProtect(ctx.internal, obj) };

        Self {
            value: JSValue::from_raw(ctx, obj),
        }
    }

    pub fn new(ctx: &'a JSContext) -> Self {
        let obj = unsafe {
            ctx.lib.ultralight().JSObjectMake(
                ctx.internal,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };

        Self {
            value: JSValue::from_raw(ctx, obj),
        }
    }

    pub fn into_value(self) -> JSValue<'a> {
        self.value
    }

    pub fn new_function_with_callback<F>(ctx: &'a JSContext, callback: F) -> Self
    where
        for<'c> F:
            FnMut(&'c JSContext, &JSObject<'c>, &[JSValue<'c>]) -> Result<JSValue<'c>, JSValue<'c>>,
    {
        LIBRARY.get_or_init(|| ctx.lib.clone());

        unsafe extern "C" fn finalize<Env>(function: ul_sys::JSObjectRef)
        where
            for<'c> Env: FnMut(
                &'c JSContext,
                &JSObject<'c>,
                &[JSValue<'c>],
            ) -> Result<JSValue<'c>, JSValue<'c>>,
        {
            let _guard = scopeguard::guard_on_unwind((), |()| {
                ::std::process::abort();
            });

            let lib = ffi_unwrap!(LIBRARY.get(), "library undefined ptr",);

            let private_data = lib.ultralight().JSObjectGetPrivate(function) as *mut Box<Env>;

            let _ = Box::from_raw(private_data);
        }

        unsafe extern "C" fn trampoline<Env>(
            ctx: ul_sys::JSContextRef,
            function: ul_sys::JSObjectRef,
            this_object: ul_sys::JSObjectRef,
            argument_count: usize,
            arguments: *const ul_sys::JSValueRef,
            exception: *mut ul_sys::JSValueRef,
        ) -> ul_sys::JSValueRef
        where
            for<'c> Env: FnMut(
                &'c JSContext,
                &JSObject<'c>,
                &[JSValue<'c>],
            ) -> Result<JSValue<'c>, JSValue<'c>>,
        {
            let _guard = scopeguard::guard_on_unwind((), |()| {
                ::std::process::abort();
            });

            let lib = ffi_unwrap!(LIBRARY.get(), "library undefined ptr",);

            let private_data = lib.ultralight().JSObjectGetPrivate(function) as *mut Box<Env>;
            let callback: &mut Box<Env> = ffi_unwrap!(private_data.as_mut(), "null ptr",);

            let ctx = JSContext::copy_from_raw(lib.clone(), ctx);
            let this = JSObject::copy_from_raw(&ctx, this_object);
            let args = std::slice::from_raw_parts(arguments, argument_count)
                .iter()
                .map(|v| JSValue::copy_from_raw(&ctx, *v))
                .collect::<Vec<_>>();

            let ret = callback(&ctx, &this, &args);
            match ret {
                Ok(value) => value.into_raw(),
                Err(value) => {
                    if !exception.is_null() {
                        *exception = value.into_raw();
                    }
                    std::ptr::null_mut()
                }
            }
        }

        let c_callback: ul_sys::JSObjectCallAsFunctionCallback = Some(trampoline::<F>);
        let callback_data: *mut Box<F> = Box::into_raw(Box::new(Box::new(callback)));

        let class_def = ul_sys::JSClassDefinition {
            finalize: Some(finalize::<F>),
            callAsFunction: c_callback,
            ..super::class::EMPTY_CLASS_DEF
        };

        let obj = unsafe {
            let class = ctx.lib.ultralight().JSClassCreate(&class_def);

            let obj = ctx
                .lib
                .ultralight()
                .JSObjectMake(ctx.internal, class, callback_data as _);

            ctx.lib.ultralight().JSClassRelease(class);

            obj
        };

        Self {
            value: JSValue::from_raw(ctx, obj),
        }
    }

    pub fn new_function(
        ctx: &'a JSContext,
        name: Option<&str>,
        param_names: &[&str],
        body: &str,
        source_url: Option<&str>,
        starting_line_number: i32,
    ) -> Result<Self, JSValue<'a>> {
        let mut exception = std::ptr::null();
        let name = name.map(|n| JSString::new(ctx.lib.clone(), n));

        let param_names: Vec<_> = param_names
            .iter()
            .map(|n| JSString::new(ctx.lib.clone(), n))
            .collect();
        let params_ptrs: Vec<_> = param_names.iter().map(|n| n.internal).collect();

        let body = JSString::new(ctx.lib.clone(), body);
        let source_url = source_url.map(|s| JSString::new(ctx.lib.clone(), s));

        let obj = unsafe {
            ctx.lib.ultralight().JSObjectMakeFunction(
                ctx.internal,
                name.map_or(std::ptr::null_mut(), |n| n.internal),
                param_names.len() as _,
                if param_names.is_empty() {
                    std::ptr::null()
                } else {
                    params_ptrs.as_ptr()
                },
                body.internal,
                source_url.map_or(std::ptr::null_mut(), |s| s.internal),
                starting_line_number,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(ctx, exception))
        } else if obj.is_null() {
            Err(JSValue::new_string(ctx, "Failed to create function"))
        } else {
            Ok(Self {
                value: JSValue::from_raw(ctx, obj),
            })
        }
    }

    pub fn new_array(ctx: &'a JSContext, items: &[JSValue]) -> Result<Self, JSValue<'a>> {
        let items_ptrs: Vec<_> = items.iter().map(|v| v.internal).collect();

        let mut exception = std::ptr::null();

        let result = unsafe {
            ctx.lib.ultralight().JSObjectMakeArray(
                ctx.internal,
                items.len() as _,
                items_ptrs.as_ptr(),
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(ctx, exception))
        } else if result.is_null() {
            Err(JSValue::new_string(ctx, "Failed to create array"))
        } else {
            Ok(Self {
                value: JSValue::from_raw(ctx, result),
            })
        }
    }

    pub fn is_function(&self) -> bool {
        unsafe {
            self.value
                .ctx
                .lib
                .ultralight()
                .JSObjectIsFunction(self.value.ctx.internal, self.value.internal as _)
        }
    }

    pub fn is_constructor(&self) -> bool {
        unsafe {
            self.value
                .ctx
                .lib
                .ultralight()
                .JSObjectIsConstructor(self.value.ctx.internal, self.value.internal as _)
        }
    }

    pub fn call_as_function(
        &self,
        this: Option<&JSObject>,
        args: &[JSValue],
    ) -> Result<JSValue, JSValue> {
        let mut exception = std::ptr::null();

        let args: Vec<_> = args.iter().map(|v| v.internal).collect();

        let result_raw = unsafe {
            self.value.ctx.lib.ultralight().JSObjectCallAsFunction(
                self.value.ctx.internal,
                self.value.internal as _,
                this.map_or(std::ptr::null_mut(), |v| v.internal as _),
                args.len(),
                args.as_ptr(),
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else if result_raw.is_null() {
            Err(JSValue::new_string(
                self.value.ctx,
                "Failed to call function",
            ))
        } else {
            Ok(JSValue::from_raw(self.value.ctx, result_raw))
        }
    }

    pub fn call_as_constructor(&self, args: &[JSValue]) -> Result<JSObject, JSValue> {
        let mut exception = std::ptr::null();

        let args: Vec<_> = args.iter().map(|v| v.internal).collect();

        let result_raw = unsafe {
            self.value.ctx.lib.ultralight().JSObjectCallAsConstructor(
                self.value.ctx.internal,
                self.value.internal as _,
                args.len(),
                args.as_ptr(),
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else if result_raw.is_null() {
            Err(JSValue::new_string(
                self.value.ctx,
                "Failed to call constructor",
            ))
        } else {
            Ok(JSObject::copy_from_raw(self.value.ctx, result_raw))
        }
    }
}

impl<'a> JSObject<'a> {
    pub fn get_property(&self, name: &str) -> Result<JSValue, JSValue> {
        let name = JSString::new(self.ctx.lib.clone(), name);
        let mut exception = std::ptr::null();

        let result_raw = unsafe {
            self.ctx.lib.ultralight().JSObjectGetProperty(
                self.ctx.internal,
                self.internal as _,
                name.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result_raw.is_null() {
            Err(JSValue::new_string(self.ctx, "Failed to get property"))
        } else {
            Ok(JSValue::from_raw(self.ctx, result_raw))
        }
    }

    pub fn get_property_at_index(&self, index: u32) -> Result<JSValue, JSValue> {
        let mut exception = std::ptr::null();

        let result_raw = unsafe {
            self.ctx.lib.ultralight().JSObjectGetPropertyAtIndex(
                self.ctx.internal,
                self.internal as _,
                index,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result_raw.is_null() {
            Err(JSValue::new_string(self.ctx, "Failed to get property"))
        } else {
            Ok(JSValue::from_raw(self.ctx, result_raw))
        }
    }

    pub fn set_property(
        &self,
        name: &str,
        value: &JSValue,
        attributes: JSPropertyAttributes,
    ) -> Result<(), JSValue> {
        let name = JSString::new(self.ctx.lib.clone(), name);
        let mut exception = std::ptr::null();

        unsafe {
            self.ctx.lib.ultralight().JSObjectSetProperty(
                self.ctx.internal,
                self.internal as _,
                name.internal,
                value.internal,
                attributes.to_raw(),
                &mut exception,
            );
        }

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(())
        }
    }

    pub fn set_property_at_index(&self, index: u32, value: &JSValue) -> Result<(), JSValue> {
        let mut exception = std::ptr::null();

        unsafe {
            self.ctx.lib.ultralight().JSObjectSetPropertyAtIndex(
                self.ctx.internal,
                self.internal as _,
                index,
                value.internal,
                &mut exception,
            );
        }

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(())
        }
    }

    pub fn get_property_names(&self) -> JSPropertyNameArray {
        let names = unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSObjectCopyPropertyNames(self.ctx.internal, self.internal as _)
        };

        JSPropertyNameArray::from_raw(self.ctx, names)
    }

    pub fn has_property(&self, name: &str) -> bool {
        let name = JSString::new(self.ctx.lib.clone(), name);

        unsafe {
            self.ctx.lib.ultralight().JSObjectHasProperty(
                self.ctx.internal,
                self.internal as _,
                name.internal,
            )
        }
    }

    pub fn delete_property(&self, name: &str) -> Result<(), JSValue> {
        let name = JSString::new(self.ctx.lib.clone(), name);
        let mut exception = std::ptr::null();

        unsafe {
            self.ctx.lib.ultralight().JSObjectDeleteProperty(
                self.ctx.internal,
                self.internal as _,
                name.internal,
                &mut exception,
            );
        }

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(())
        }
    }

    pub fn get_property_for_key(&self, key: &JSValue) -> Result<JSValue, JSValue> {
        let mut exception = std::ptr::null();

        let result_raw = unsafe {
            self.ctx.lib.ultralight().JSObjectGetPropertyForKey(
                self.ctx.internal,
                self.internal as _,
                key.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else if result_raw.is_null() {
            Err(JSValue::new_string(self.ctx, "Failed to get property"))
        } else {
            Ok(JSValue::from_raw(self.ctx, result_raw))
        }
    }

    pub fn set_property_for_key(
        &self,
        key: &JSValue,
        value: &JSValue,
        attributes: JSPropertyAttributes,
    ) -> Result<(), JSValue> {
        let mut exception = std::ptr::null();

        unsafe {
            self.ctx.lib.ultralight().JSObjectSetPropertyForKey(
                self.ctx.internal,
                self.internal as _,
                key.internal,
                value.internal,
                attributes.to_raw(),
                &mut exception,
            );
        }

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(())
        }
    }

    pub fn has_property_for_key(&self, key: &JSValue) -> Result<bool, JSValue> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSObjectHasPropertyForKey(
                self.ctx.internal,
                self.internal as _,
                key.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(result)
        }
    }

    pub fn delete_property_for_key(&self, key: &JSValue) -> Result<bool, JSValue> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.ctx.lib.ultralight().JSObjectDeletePropertyForKey(
                self.ctx.internal,
                self.internal as _,
                key.internal,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.ctx, exception))
        } else {
            Ok(result)
        }
    }
}

impl<'a> AsRef<JSValue<'a>> for JSObject<'a> {
    fn as_ref(&self) -> &JSValue<'a> {
        &self.value
    }
}

impl<'a> Deref for JSObject<'a> {
    type Target = JSValue<'a>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct JSPropertyNameArray<'a> {
    internal: ul_sys::JSPropertyNameArrayRef,
    ctx: &'a JSContext,
}

impl<'a> JSPropertyNameArray<'a> {
    pub(crate) fn from_raw(ctx: &'a JSContext, array: ul_sys::JSPropertyNameArrayRef) -> Self {
        assert!(!array.is_null());

        Self {
            internal: array,
            ctx,
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSPropertyNameArrayGetCount(self.internal)
        }
    }

    pub fn get(&self, index: usize) -> JSString {
        let name = unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSPropertyNameArrayGetNameAtIndex(self.internal, index)
        };

        JSString::copy_from_raw(self.ctx.lib.clone(), name)
    }

    pub fn into_vec(self) -> Vec<String> {
        self.into()
    }
}

impl From<JSPropertyNameArray<'_>> for Vec<String> {
    fn from(array: JSPropertyNameArray) -> Self {
        let mut names = Vec::with_capacity(array.len());

        for i in 0..array.len() {
            let name = array.get(i).to_string();
            names.push(name);
        }

        names
    }
}

impl Drop for JSPropertyNameArray<'_> {
    fn drop(&mut self) {
        unsafe {
            self.ctx
                .lib
                .ultralight()
                .JSPropertyNameArrayRelease(self.internal);
        }
    }
}
