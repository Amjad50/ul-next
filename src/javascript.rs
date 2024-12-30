//! JavaScriptCore bindings for Ultralight.
//!
//! This module provides a high-level API for working with JavaScriptCore in
//! Ultralight. It is a thin wrapper around the C API provided by JavaScriptCore.
//!
//! # Example
//! ```rust,no_run
//! # use ul_next::javascript::*;
//! # let context: JSContext = unsafe {std::mem::zeroed()};
//! # let lib: std::sync::Arc<ul_next::Library> = unsafe {std::mem::zeroed()};
//!
//! let object = context.global_object();
//! let string = JSValue::new_string(&context, "Hello, world!");
//! object.set_property("greeting", &string, JSPropertyAttributes::default());
//!
//! // this can be called from JavaScript
//! let rust_callback = JSObject::new_function_with_callback(&context, |ctx, this, args| {
//!     Ok(JSValue::new_string(ctx, "Hello from Rust!"))
//! });
//! object.set_property("rustCallback", &rust_callback, JSPropertyAttributes::default());
//! ```

mod class;
mod context;
mod object;
mod string;
mod typed_array;
mod value;

pub use context::JSContext;
pub use object::{JSObject, JSPropertyAttributes, JSPropertyNameArray};
pub use string::JSString;
pub use typed_array::{JSTypedArray, JSTypedArrayType};
pub use value::{AsJSValue, JSType, JSValue};
