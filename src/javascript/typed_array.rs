use std::ops::Deref;

use super::{AsJSValue, JSContext, JSValue};

/// An enum identifying the Typed Array type of a [`JSTypedArray`].
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JSTypedArrayType {
    Int8Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt8Array,
    Int16Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt16Array,
    Int32Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt32Array,
    Uint8Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint8Array,
    Uint8ClampedArray = ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint8ClampedArray,
    Uint16Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint16Array,
    Uint32Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint32Array,
    Float32Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeFloat32Array,
    Float64Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeFloat64Array,
    ArrayBuffer = ul_sys::JSTypedArrayType_kJSTypedArrayTypeArrayBuffer,
    None = ul_sys::JSTypedArrayType_kJSTypedArrayTypeNone,
    BigInt64Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeBigInt64Array,
    BigUint64Array = ul_sys::JSTypedArrayType_kJSTypedArrayTypeBigUint64Array,
}

/// A JavaScript Typed Array object.
#[derive(Clone, Debug)]
pub struct JSTypedArray<'a> {
    pub(crate) value: JSValue<'a>,
}

impl<'a> JSTypedArray<'a> {
    /// Creates a Javascript Typed Array object with the given number of elements
    /// all initialized to zero.
    ///
    /// Returns [`Err`] if an exception occurred while creating the object.
    pub fn new(
        ctx: &'a JSContext,
        array_type: JSTypedArrayType,
        length: usize,
    ) -> Result<Self, JSValue<'a>> {
        let mut exception = std::ptr::null();
        let value = unsafe {
            ctx.lib.ultralight().JSObjectMakeTypedArray(
                ctx.internal,
                array_type as _,
                length,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(ctx, exception))
        } else if value.is_null() {
            return Err(JSValue::new_string(ctx, "Failed to create typed array"));
        } else {
            Ok(Self {
                value: JSValue::from_raw(ctx, value),
            })
        }
    }

    /// Creates a JavaScript Typed Array object from existing bytes buffer.
    ///
    /// This will copy the bytes and manages its memory.
    ///
    /// Returns [`Err`] if an exception occurred while creating the object.
    pub fn new_copy_from_bytes(
        ctx: &'a JSContext,
        array_type: JSTypedArrayType,
        bytes: &[u8],
    ) -> Result<Self, JSValue<'a>> {
        extern "C" fn deallocator(
            bytes: *mut std::ffi::c_void,
            deallocator_context: *mut std::ffi::c_void,
        ) {
            let slice_size = deallocator_context as usize;

            drop(unsafe {
                Box::from_raw(std::slice::from_raw_parts_mut(bytes as *mut u8, slice_size))
            })
        }

        let mut exception = std::ptr::null();
        let boxed_bytes = Vec::from(bytes).into_boxed_slice();

        let date_size = boxed_bytes.len();
        let data = Box::into_raw(boxed_bytes);

        let value = unsafe {
            ctx.lib.ultralight().JSObjectMakeTypedArrayWithBytesNoCopy(
                ctx.internal,
                array_type as _,
                data as _,
                bytes.len(),
                Some(deallocator),
                date_size as _,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(ctx, exception))
        } else if value.is_null() {
            return Err(JSValue::new_string(ctx, "Failed to create typed array"));
        } else {
            Ok(Self {
                value: JSValue::from_raw(ctx, value),
            })
        }
    }

    /// Returns the length of a JavaScript Typed Array object.
    ///
    /// Returns [`Err`] if an exception occurred while getting the length.
    pub fn len(&self) -> Result<usize, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.value.ctx.lib.ultralight().JSObjectGetTypedArrayLength(
                self.value.ctx.internal,
                self.value.internal as _,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else {
            Ok(result)
        }
    }

    /// Returns whether the JavaScript Typed Array object is empty.
    ///
    /// Returns [`Err`] if an exception occurred while getting the length.
    pub fn is_empty(&self) -> Result<bool, JSValue<'a>> {
        self.len().map(|len| len == 0)
    }

    /// Returns the byte length of a JavaScript Typed Array object.
    ///
    /// Returns [`Err`] if an exception occurred while getting the byte length.
    pub fn byte_length(&self) -> Result<usize, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.value
                .ctx
                .lib
                .ultralight()
                .JSObjectGetTypedArrayByteLength(
                    self.value.ctx.internal,
                    self.value.internal as _,
                    &mut exception,
                )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else {
            Ok(result)
        }
    }

    /// Returns the byte offset of a JavaScript Typed Array object.
    ///
    /// Returns [`Err`] if an exception occurred while getting the byte offset.
    pub fn byte_offset(&self) -> Result<usize, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let result = unsafe {
            self.value
                .ctx
                .lib
                .ultralight()
                .JSObjectGetTypedArrayByteOffset(
                    self.value.ctx.internal,
                    self.value.internal as _,
                    &mut exception,
                )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else {
            Ok(result)
        }
    }

    /// Returns the type of a JavaScript Typed Array object.
    ///
    /// Returns [`Err`] if an exception occurred while getting the type,
    /// or [`JSTypedArrayType::None`] if object isn't a Typed Array.
    pub fn ty(&self) -> Result<JSTypedArrayType, JSValue<'a>> {
        let mut exception = std::ptr::null();

        let ty = unsafe {
            self.value.ctx.lib.ultralight().JSValueGetTypedArrayType(
                self.value.ctx.internal,
                self.value.internal as _,
                &mut exception,
            )
        };

        if !exception.is_null() {
            Err(JSValue::from_raw(self.value.ctx, exception))
        } else {
            match ty {
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt8Array => {
                    Ok(JSTypedArrayType::Int8Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt16Array => {
                    Ok(JSTypedArrayType::Int16Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeInt32Array => {
                    Ok(JSTypedArrayType::Int32Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint8Array => {
                    Ok(JSTypedArrayType::Uint8Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint8ClampedArray => {
                    Ok(JSTypedArrayType::Uint8ClampedArray)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint16Array => {
                    Ok(JSTypedArrayType::Uint16Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeUint32Array => {
                    Ok(JSTypedArrayType::Uint32Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeFloat32Array => {
                    Ok(JSTypedArrayType::Float32Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeFloat64Array => {
                    Ok(JSTypedArrayType::Float64Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeArrayBuffer => {
                    Ok(JSTypedArrayType::ArrayBuffer)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeNone => Ok(JSTypedArrayType::None),
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeBigInt64Array => {
                    Ok(JSTypedArrayType::BigInt64Array)
                }
                ul_sys::JSTypedArrayType_kJSTypedArrayTypeBigUint64Array => {
                    Ok(JSTypedArrayType::BigUint64Array)
                }
                _ => Err(JSValue::new_string(
                    self.value.ctx,
                    &format!("Unknown typed array type: {}", ty),
                )),
            }
        }
    }
}

impl<'a> AsRef<JSValue<'a>> for JSTypedArray<'a> {
    fn as_ref(&self) -> &JSValue<'a> {
        &self.value
    }
}

impl<'a> Deref for JSTypedArray<'a> {
    type Target = JSValue<'a>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> AsJSValue<'a> for JSTypedArray<'a> {
    fn into_value(self) -> JSValue<'a> {
        self.value
    }

    fn as_value(&self) -> &JSValue<'a> {
        &self.value
    }
}
